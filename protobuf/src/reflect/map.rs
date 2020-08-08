use std::collections::hash_map;
use std::collections::HashMap;
use std::hash::Hash;

use crate::reflect::reflect_eq::ReflectEq;
use crate::reflect::reflect_eq::ReflectEqMode;
use crate::reflect::runtime_type_dynamic::RuntimeTypeDynamic;
use crate::reflect::value::ProtobufValue;
use crate::reflect::ProtobufValueSized;
use crate::reflect::ReflectValueBox;
use crate::reflect::ReflectValueRef;

/// Implemented for `HashMap` with appropriate keys and values
pub(crate) trait ReflectMap: Send + Sync + 'static {
    fn reflect_iter(&self) -> ReflectMapIter;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;

    fn get(&self, key: ReflectValueRef) -> Option<&dyn ProtobufValue>;

    fn insert(&mut self, key: ReflectValueBox, value: ReflectValueBox);

    fn clear(&mut self);

    fn key_type(&self) -> &'static dyn RuntimeTypeDynamic;

    fn value_type(&self) -> &'static dyn RuntimeTypeDynamic;
}

impl<K: ProtobufValueSized + Eq + Hash, V: ProtobufValueSized> ReflectMap for HashMap<K, V> {
    fn reflect_iter<'a>(&'a self) -> ReflectMapIter<'a> {
        ReflectMapIter {
            imp: Box::new(ReflectMapIterImpl::<'a, K, V> { iter: self.iter() }),
        }
    }

    fn len(&self) -> usize {
        HashMap::len(self)
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn get(&self, key: ReflectValueRef) -> Option<&dyn ProtobufValue> {
        // TODO: malloc for string or bytes
        let key: K = key.to_box().downcast().expect("wrong key type");
        self.get(&key).map(|v| v as &dyn ProtobufValue)
    }

    fn insert(&mut self, key: ReflectValueBox, value: ReflectValueBox) {
        let key: K = key.downcast().expect("wrong key type");
        let value: V = value.downcast().expect("wrong value type");
        self.insert(key, value);
    }

    fn clear(&mut self) {
        self.clear();
    }

    fn key_type(&self) -> &'static dyn RuntimeTypeDynamic {
        K::dynamic()
    }

    fn value_type(&self) -> &'static dyn RuntimeTypeDynamic {
        V::dynamic()
    }
}

trait ReflectMapIterTrait<'a> {
    fn next(&mut self) -> Option<(&'a dyn ProtobufValue, &'a dyn ProtobufValue)>;
    fn key_type(&self) -> &'static dyn RuntimeTypeDynamic;
    fn value_type(&self) -> &'static dyn RuntimeTypeDynamic;
}

struct ReflectMapIterImpl<'a, K: Eq + Hash + 'static, V: 'static> {
    iter: hash_map::Iter<'a, K, V>,
}

impl<'a, K: ProtobufValueSized + Eq + Hash, V: ProtobufValueSized> ReflectMapIterTrait<'a>
    for ReflectMapIterImpl<'a, K, V>
{
    fn next(&mut self) -> Option<(&'a dyn ProtobufValue, &'a dyn ProtobufValue)> {
        match self.iter.next() {
            Some((k, v)) => Some((k as &dyn ProtobufValue, v as &dyn ProtobufValue)),
            None => None,
        }
    }

    fn key_type(&self) -> &'static dyn RuntimeTypeDynamic {
        K::dynamic()
    }

    fn value_type(&self) -> &'static dyn RuntimeTypeDynamic {
        V::dynamic()
    }
}

pub struct ReflectMapIter<'a> {
    imp: Box<dyn ReflectMapIterTrait<'a> + 'a>,
}

impl<'a> Iterator for ReflectMapIter<'a> {
    type Item = (&'a dyn ProtobufValue, &'a dyn ProtobufValue);

    fn next(&mut self) -> Option<(&'a dyn ProtobufValue, &'a dyn ProtobufValue)> {
        self.imp.next()
    }
}

impl<'a> IntoIterator for &'a dyn ReflectMap {
    type Item = (&'a dyn ProtobufValue, &'a dyn ProtobufValue);
    type IntoIter = ReflectMapIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.reflect_iter()
    }
}

/// Dynamic reference to `map` field
#[derive(Copy, Clone)]
pub struct ReflectMapRef<'a> {
    pub(crate) map: &'a dyn ReflectMap,
}

/// Dynamic mutable reference to `map` field
pub struct ReflectMapMut<'a> {
    pub(crate) map: &'a mut dyn ReflectMap,
}

impl<'a> ReflectMapRef<'a> {
    /// Size of the map
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Is map empty?
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Find a value by given key.
    pub fn get(&self, key: ReflectValueRef) -> Option<ReflectValueRef> {
        self.map.get(key).map(|v| self.value_type().value_to_ref(v))
    }

    /// Map key type
    pub fn key_type(&self) -> &dyn RuntimeTypeDynamic {
        self.map.key_type()
    }

    /// Map value type
    pub fn value_type(&self) -> &dyn RuntimeTypeDynamic {
        self.map.value_type()
    }
}

impl<'a> ReflectEq for ReflectMapRef<'a> {
    fn reflect_eq(&self, that: &Self, mode: &ReflectEqMode) -> bool {
        let len = self.len();

        if len != that.len() {
            return false;
        }

        for (k, va) in self {
            let vb = match that.get(k) {
                Some(v) => v,
                None => return false,
            };

            if !va.reflect_eq(&vb, mode) {
                return false;
            }
        }

        true
    }
}

impl<'a> ReflectMapMut<'a> {
    fn as_ref(&'a self) -> ReflectMapRef<'a> {
        ReflectMapRef { map: self.map }
    }

    /// Map key type
    pub fn key_type(&self) -> &dyn RuntimeTypeDynamic {
        self.map.key_type()
    }

    /// Map value type
    pub fn value_type(&self) -> &dyn RuntimeTypeDynamic {
        self.map.value_type()
    }

    /// Number of map entries
    pub fn len(&self) -> usize {
        self.as_ref().len()
    }

    /// Is this map empty?
    pub fn is_empty(&self) -> bool {
        self.as_ref().is_empty()
    }

    /// Find a value for given key
    pub fn get(&self, key: ReflectValueRef) -> Option<ReflectValueRef> {
        self.map.get(key).map(|v| self.value_type().value_to_ref(v))
    }

    /// Insert a value into the map
    pub fn insert(&mut self, key: ReflectValueBox, value: ReflectValueBox) {
        self.map.insert(key, value)
    }

    /// Clear
    pub fn clear(&mut self) {
        self.map.clear();
    }
}

/// Iterator over map
pub struct ReflectMapRefIter<'a> {
    iter: ReflectMapIter<'a>,
}

impl<'a> ReflectMapRefIter<'a> {
    fn key_type(&self) -> &'static dyn RuntimeTypeDynamic {
        self.iter.imp.key_type()
    }

    fn value_type(&self) -> &'static dyn RuntimeTypeDynamic {
        self.iter.imp.value_type()
    }
}

impl<'a> Iterator for ReflectMapRefIter<'a> {
    type Item = (ReflectValueRef<'a>, ReflectValueRef<'a>);

    fn next(&mut self) -> Option<(ReflectValueRef<'a>, ReflectValueRef<'a>)> {
        self.iter.next().map(|(k, v)| {
            (
                self.key_type().value_to_ref(k),
                self.value_type().value_to_ref(v),
            )
        })
    }
}

impl<'a, 'b> IntoIterator for &'b ReflectMapRef<'a> {
    type Item = (ReflectValueRef<'a>, ReflectValueRef<'a>);
    type IntoIter = ReflectMapRefIter<'a>;

    fn into_iter(self) -> ReflectMapRefIter<'a> {
        ReflectMapRefIter {
            iter: self.map.reflect_iter(),
        }
    }
}
