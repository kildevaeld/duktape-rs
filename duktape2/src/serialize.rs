use serde::{Deserialize, de};
use super::error::{DukResult, DukError};
use super::context::{Context, Idx, Type};
use super::to_context::ToDuktapeContext;
use super::from_context::FromDuktapeContext;
use super::object::{Object, JSObject, ObjectIterator};
use super::array::{Array, JSArray};
use super::reference::{JSValue, Reference};



pub struct Deserializer<'a> {
    ctx: &'a Context,
    idx:Idx,
}


impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'a> {
    type Error = DukError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let t = self.ctx.get_type(self.idx);
    
        match t {
            Type::Null | Type::Undefined => visitor.visit_unit(),
            Type::Boolean => self.deserialize_bool(visitor),
            Type::String => self.deserialize_string(visitor),
            Type::Buffer => visitor.visit_bytes(self.ctx.get_bytes(self.idx)?),
            Type::Number => visitor.visit_f64(self.ctx.get_number(self.idx)?),
            //Type::Object => visitor.visit_map(map: A)
            _ => {
                unimplemented!("cannot deserialize {:?}", t)
            } 
        }

    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_bool(self.ctx.get(self.idx)?)
    }

    // The `parse_signed` function is generic over the integer type `T` so here
    // it is invoked with `T=i8`. The next 8 methods are similar.
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i8(self.ctx.get(self.idx)?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        
        visitor.visit_i16(self.ctx.get(self.idx)?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i32(self.ctx.get(self.idx)?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i64(self.ctx.get(self.idx)?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u8(self.ctx.get(self.idx)?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u16(self.ctx.get(self.idx)?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u32(self.ctx.get(self.idx)?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u64(self.ctx.get(self.idx)?)
    }

    // Float parsing is stupidly hard.
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f32(self.ctx.get(self.idx)?)
    }

    // Float parsing is stupidly hard.
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f64(self.ctx.get(self.idx)?)
    }


    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_str(self.ctx.get_string(self.idx)?)

    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    // The `Serializer` implementation on the previous page serialized byte
    // arrays as JSON arrays of bytes. Handle that representation here.
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_bytes(self.ctx.get_bytes(self.idx)?)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_byte_buf(self.ctx.get_bytes(self.idx)?.to_vec())   
    }


    serde::forward_to_deserialize_any! {
        char option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }

    // i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 str string bytes byte_buf

}


pub struct MapAccess<'a> {
    o: ObjectIterator<'a>,
    item: Option<(&'a str, Reference<'a>)>,
}


impl<'a> MapAccess<'a> {
    pub fn new(o: ObjectIterator<'a>) -> MapAccess {
        MapAccess{o,item:None}
    }
}

impl<'de, 'a> de::MapAccess<'de> for MapAccess<'a> {
    type Error = DukError;


    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de> {
        self.item = self.o.next();

        if self.item.is_none() {
            return Ok(None);
        }

        seed.deserialize(deserializer: D)
    }


    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {

    }

}


pub fn from_str<'a, T>(s: &'a Context, idx: Idx) -> DukResult<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer{ctx:s, idx};
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
}


#[test]
fn test_enum() {
    // #[derive(Deserialize, PartialEq, Debug)]
    // enum E {
    //     Unit,
    //     Newtype(u32),
    //     Tuple(u32, u32),
    //     Struct { a: u32 },
    // }

    let ctx = Context::new().unwrap();

    ctx.push_null();
    from_str::<()>(&ctx, -1).unwrap();
    ctx.pop(1);

    ctx.push_string("Hello, World!");
    assert_eq!("Hello, World!", from_str::<String>(&ctx, -1).unwrap());
    ctx.pop(1);


    ctx.push_boolean(true);
    assert_eq!(true, from_str::<bool>(&ctx, -1).unwrap());
    ctx.pop(1);

    ctx.push_boolean(false);
    assert_eq!(false, from_str::<bool>(&ctx, -1).unwrap());
    ctx.pop(1);

    ctx.push_int(101);
    assert_eq!(101, from_str::<i32>(&ctx, -1).unwrap());
    ctx.pop(1);

    ctx.push_number(101.0);
    assert_eq!(101.0, from_str::<f32>(&ctx, -1).unwrap());
    ctx.pop(1);


    
}


