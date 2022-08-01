use super::error::{Result, SerdeError as Error};

use serde::{
    ser::{
        SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant, Serializer,
    },
    Serialize,
};

pub struct OrmSerializer {
    pub args: Vec<rdbc::Arg>,
    current_placeholder: Option<rdbc::Placeholder>,
    current_index: u64,
    level: u64,
    sub_json: String,
    serialize_seq: bool,
}

impl OrmSerializer {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for OrmSerializer {
    fn default() -> Self {
        Self {
            args: Default::default(),
            current_placeholder: Default::default(),
            current_index: 0,
            level: 0,
            sub_json: Default::default(),
            serialize_seq: false,
        }
    }
}

impl OrmSerializer {
    fn next_parameter(&mut self, placeholder: Option<rdbc::Placeholder>) -> Result<()> {
        self.current_placeholder = placeholder;

        if self.current_placeholder.is_none() {
            self.current_index += 1;
            self.current_placeholder = Some(rdbc::Placeholder::Index(self.current_index));
        }

        Ok(())
    }
}

impl<'a> Serializer for &'a mut OrmSerializer {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        if self.current_placeholder.is_none() {
            self.next_parameter(None)?;
        }

        self.args.push(rdbc::Arg {
            pos: self.current_placeholder.take().unwrap(),
            value: rdbc::Value::I64(if v { 1 } else { 0 }),
        });
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        if self.current_placeholder.is_none() {
            self.next_parameter(None)?;
        }

        self.args.push(rdbc::Arg {
            pos: self.current_placeholder.take().unwrap(),
            value: rdbc::Value::I64(v),
        });

        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        if self.serialize_seq {
            return Err(Error::SerdeBytes);
        }

        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        if self.current_placeholder.is_none() {
            self.next_parameter(None)?;
        }

        self.args.push(rdbc::Arg {
            pos: self.current_placeholder.take().unwrap(),
            value: rdbc::Value::I64(v as i64),
        });
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        if self.current_placeholder.is_none() {
            self.next_parameter(None)?;
        }

        self.args.push(rdbc::Arg {
            pos: self.current_placeholder.take().unwrap(),
            value: rdbc::Value::F64(v),
        });
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        if self.current_placeholder.is_none() {
            self.next_parameter(None)?;
        }

        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        if self.current_placeholder.is_none() {
            self.next_parameter(None)?;
        }

        self.args.push(rdbc::Arg {
            pos: self.current_placeholder.take().unwrap(),
            value: rdbc::Value::String(v.to_owned()),
        });
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        if self.current_placeholder.is_none() {
            self.next_parameter(None)?;
        }

        self.args.push(rdbc::Arg {
            pos: self.current_placeholder.take().unwrap(),
            value: rdbc::Value::Bytes(v.to_owned()),
        });
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // In Serde, unit means an anonymous value containing no data.
    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }

    // Unit struct means a named value containing no data.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    // When serializing a unit variant (or any other kind of variant), formats
    // can choose whether to keep track of it by index or by name. Binary
    // formats typically use the index of the variant and human-readable formats
    // typically use the name.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain.
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        if self.level != 0 {
            self.sub_json = "[".to_owned();
        }

        self.level += 1;
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        if self.level != 0 {
            self.sub_json = "[".to_owned();
        }

        self.level += 1;
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        if self.current_placeholder.is_none() {
            self.next_parameter(None)?;
        }

        self.sub_json = format!("{}[", variant);
        self.level += 1;
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        if self.level != 0 {
            self.sub_json = "{".to_owned();
        }

        self.level += 1;
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        if self.level != 0 {
            self.sub_json = "{".to_owned();
        }

        self.level += 1;
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        if self.current_placeholder.is_none() {
            self.next_parameter(None)?;
        }

        self.sub_json = format!("{}{{", variant);

        self.level += 1;

        Ok(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.sub_json = format!("{}[", variant);

        use serde::ser::Error;

        let json = serde_json::to_string(value).map_err(Error::custom)?;

        self.sub_json += &json;

        self.sub_json += "]";

        if self.current_placeholder.is_none() {
            self.next_parameter(None)?;
        }

        self.args.push(rdbc::Arg {
            pos: self.current_placeholder.take().unwrap(),
            value: rdbc::Value::String(self.sub_json.to_owned()),
        });

        self.sub_json.clear();

        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        if self.current_placeholder.is_none() {
            self.next_parameter(None)?;
        }

        if self.level != 0 {
            self.sub_json = "[".to_owned();
        }

        self.level += 1;
        self.serialize_seq = true;
        Ok(self)
    }
}

impl<'a> SerializeSeq for &'a mut OrmSerializer {
    // Must match the `Ok` type of the serializer.
    type Ok = ();
    // Must match the `Error` type of the serializer.
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if self.level == 1 {
            self.next_parameter(None)?;
            value.serialize(&mut **self)
        } else {
            use serde::ser::Error;
            if !self.sub_json.ends_with("[") {
                self.sub_json += ",";
            }

            let json = serde_json::to_string(value).map_err(Error::custom)?;

            self.sub_json += json.as_str();

            Ok(())
        }
    }

    // Close the sequence.
    fn end(self) -> Result<()> {
        self.level -= 1;
        self.serialize_seq = false;

        if self.level != 0 {
            self.sub_json += "]";

            self.args.push(rdbc::Arg {
                pos: self.current_placeholder.take().unwrap(),
                value: rdbc::Value::String(self.sub_json.to_owned()),
            });

            self.sub_json.clear();
        }

        Ok(())
    }
}

impl<'a> SerializeTuple for &'a mut OrmSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if self.level == 1 {
            self.next_parameter(None)?;
            value.serialize(&mut **self)
        } else {
            use serde::ser::Error;
            if !self.sub_json.ends_with("[") {
                self.sub_json += ",";
            }

            let json = serde_json::to_string(value).map_err(Error::custom)?;

            self.sub_json += json.as_str();

            Ok(())
        }
    }

    fn end(self) -> Result<()> {
        self.level -= 1;

        if self.level != 0 {
            self.sub_json += "]";

            self.args.push(rdbc::Arg {
                pos: self.current_placeholder.take().unwrap(),
                value: rdbc::Value::String(self.sub_json.to_owned()),
            });

            self.sub_json.clear();
        }

        Ok(())
    }
}

impl<'a> SerializeTupleStruct for &'a mut OrmSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if self.level == 1 {
            self.next_parameter(None)?;
            value.serialize(&mut **self)
        } else {
            use serde::ser::Error;
            if !self.sub_json.ends_with("[") {
                self.sub_json += ",";
            }

            let json = serde_json::to_string(value).map_err(Error::custom)?;

            self.sub_json += json.as_str();

            Ok(())
        }
    }

    fn end(self) -> Result<()> {
        self.level -= 1;

        if self.level != 0 {
            self.sub_json += "]";

            self.args.push(rdbc::Arg {
                pos: self.current_placeholder.take().unwrap(),
                value: rdbc::Value::String(self.sub_json.to_owned()),
            });

            self.sub_json.clear();
        }

        Ok(())
    }
}

impl<'a> SerializeTupleVariant for &'a mut OrmSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        use serde::ser::Error;
        if !self.sub_json.ends_with("[") {
            self.sub_json += ",";
        }

        let json = serde_json::to_string(value).map_err(Error::custom)?;

        self.sub_json += json.as_str();

        Ok(())
    }

    fn end(self) -> Result<()> {
        self.level -= 1;

        self.sub_json += "]";

        self.args.push(rdbc::Arg {
            pos: self.current_placeholder.take().unwrap(),
            value: rdbc::Value::String(self.sub_json.to_owned()),
        });

        self.sub_json.clear();
        Ok(())
    }
}

impl<'a> SerializeMap for &'a mut OrmSerializer {
    type Ok = ();
    type Error = Error;

    // The Serde data model allows map keys to be any serializable type. JSON
    // only allows string keys so the implementation below will produce invalid
    // JSON if the key serializes as something other than a string.
    //
    // A real JSON serializer would need to validate that map keys are strings.
    // This can be done by using a different Serializer to serialize the key
    // (instead of `&mut **self`) and having that other serializer only
    // implement `serialize_str` and return an error on any other data type.
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        use serde::ser::Error;

        let json = serde_json::to_string(key).map_err(Error::custom)?;

        if self.level != 1 {
            if !self.sub_json.ends_with("{") {
                self.sub_json += ",";
            }

            self.sub_json += json.as_str();

            self.sub_json += ":";
        } else {
            self.next_parameter(Some(rdbc::Placeholder::Name(json)))?;
        }

        Ok(())
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if self.level != 1 {
            use serde::ser::Error;

            let json = serde_json::to_string(value).map_err(Error::custom)?;

            self.sub_json += json.as_str();

            Ok(())
        } else {
            value.serialize(&mut **self)
        }
    }

    fn end(self) -> Result<()> {
        self.level -= 1;

        if self.level != 0 {
            self.sub_json += "}";

            self.args.push(rdbc::Arg {
                pos: self.current_placeholder.take().unwrap(),
                value: rdbc::Value::String(self.sub_json.to_owned()),
            });

            self.sub_json.clear();
        }
        Ok(())
    }
}

impl<'a> SerializeStruct for &'a mut OrmSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if self.level == 1 {
            log::debug!("serialize {}", key);
            self.next_parameter(Some(rdbc::Placeholder::Name(key.to_owned())))?;

            value.serialize(&mut **self)
        } else {
            use serde::ser::Error;
            if !self.sub_json.ends_with("{") {
                self.sub_json += ",";
            }

            self.sub_json += key;

            self.sub_json += ":";

            let json = serde_json::to_string(value).map_err(Error::custom)?;

            self.sub_json += json.as_str();

            Ok(())
        }
    }

    fn end(self) -> Result<()> {
        self.level -= 1;

        if self.level != 0 {
            self.sub_json += "}";

            self.args.push(rdbc::Arg {
                pos: self.current_placeholder.take().unwrap(),
                value: rdbc::Value::String(self.sub_json.to_owned()),
            });

            self.sub_json.clear();
        }

        Ok(())
    }
}

impl<'a> SerializeStructVariant for &'a mut OrmSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.sub_json.ends_with("{") {
            self.sub_json += ",";
        }

        self.sub_json += key;

        self.sub_json += ":";

        use serde::ser::Error;

        let json = serde_json::to_string(value).map_err(Error::custom)?;

        self.sub_json += json.as_str();

        Ok(())
    }

    fn end(self) -> Result<()> {
        self.level -= 1;

        self.sub_json += "}";

        self.args.push(rdbc::Arg {
            pos: self.current_placeholder.take().unwrap(),
            value: rdbc::Value::String(self.sub_json.to_owned()),
        });

        self.sub_json.clear();
        Ok(())
    }
}
