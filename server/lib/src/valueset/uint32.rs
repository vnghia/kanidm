use crate::prelude::*;
use crate::schema::SchemaAttribute;
use crate::valueset::{
    DbValueSetV2, ScimResolveStatus, ValueSet, ValueSetResolveStatus, ValueSetScimPut,
};
use kanidm_proto::scim_v1::JsonValue;
use smolset::SmolSet;

#[derive(Debug, Clone)]
pub struct ValueSetUint32 {
    set: SmolSet<[u32; 1]>,
}

impl ValueSetUint32 {
    pub fn new(b: u32) -> Box<Self> {
        let mut set = SmolSet::new();
        set.insert(b);
        Box::new(ValueSetUint32 { set })
    }

    pub fn push(&mut self, b: u32) -> bool {
        self.set.insert(b)
    }

    pub fn from_dbvs2(data: Vec<u32>) -> Result<ValueSet, OperationError> {
        let set = data.into_iter().collect();
        Ok(Box::new(ValueSetUint32 { set }))
    }

    // We need to allow this, because rust doesn't allow us to impl FromIterator on foreign
    // types, and u32 is foreign.
    #[allow(clippy::should_implement_trait)]
    pub fn from_iter<T>(iter: T) -> Option<Box<Self>>
    where
        T: IntoIterator<Item = u32>,
    {
        let set = iter.into_iter().collect();
        Some(Box::new(ValueSetUint32 { set }))
    }
}

impl ValueSetScimPut for ValueSetUint32 {
    fn from_scim_json_put(value: JsonValue) -> Result<ValueSetResolveStatus, OperationError> {
        let value: u32 = serde_json::from_value(value).map_err(|err| {
            error!(?err, "SCIM uint32 syntax invalid");
            OperationError::SC0006Uint32SyntaxInvalid
        })?;

        let mut set = SmolSet::new();
        set.insert(value);

        Ok(ValueSetResolveStatus::Resolved(Box::new(ValueSetUint32 {
            set,
        })))
    }
}

impl ValueSetT for ValueSetUint32 {
    fn insert_checked(&mut self, value: Value) -> Result<bool, OperationError> {
        match value {
            Value::Uint32(u) => Ok(self.set.insert(u)),
            _ => {
                debug_assert!(false);
                Err(OperationError::InvalidValueState)
            }
        }
    }

    fn clear(&mut self) {
        self.set.clear();
    }

    fn remove(&mut self, pv: &PartialValue, _cid: &Cid) -> bool {
        match pv {
            PartialValue::Uint32(u) => self.set.remove(u),
            _ => {
                debug_assert!(false);
                true
            }
        }
    }

    fn contains(&self, pv: &PartialValue) -> bool {
        match pv {
            PartialValue::Uint32(u) => self.set.contains(u),
            _ => false,
        }
    }

    fn substring(&self, _pv: &PartialValue) -> bool {
        false
    }

    fn startswith(&self, _pv: &PartialValue) -> bool {
        false
    }

    fn endswith(&self, _pv: &PartialValue) -> bool {
        false
    }

    fn lessthan(&self, pv: &PartialValue) -> bool {
        match pv {
            PartialValue::Uint32(u) => self.set.iter().any(|i| i < u),
            _ => false,
        }
    }

    fn len(&self) -> usize {
        self.set.len()
    }

    fn generate_idx_eq_keys(&self) -> Vec<String> {
        self.set.iter().map(|b| b.to_string()).collect()
    }

    fn syntax(&self) -> SyntaxType {
        SyntaxType::Uint32
    }

    fn validate(&self, _schema_attr: &SchemaAttribute) -> bool {
        true
    }

    fn to_proto_string_clone_iter(&self) -> Box<dyn Iterator<Item = String> + '_> {
        Box::new(self.set.iter().map(|b| b.to_string()))
    }

    fn to_scim_value(&self) -> Option<ScimResolveStatus> {
        if self.len() == 1 {
            // Because self.len == 1 we know this has to yield a value.
            let b = self.set.iter().copied().next().unwrap_or_default();
            Some(b.into())
        } else {
            // Nothing is MV for this today
            None
        }
    }

    fn to_db_valueset_v2(&self) -> DbValueSetV2 {
        DbValueSetV2::Uint32(self.set.iter().cloned().collect())
    }

    fn to_partialvalue_iter(&self) -> Box<dyn Iterator<Item = PartialValue> + '_> {
        Box::new(self.set.iter().copied().map(PartialValue::new_uint32))
    }

    fn to_value_iter(&self) -> Box<dyn Iterator<Item = Value> + '_> {
        Box::new(self.set.iter().copied().map(Value::new_uint32))
    }

    fn equal(&self, other: &ValueSet) -> bool {
        if let Some(other) = other.as_uint32_set() {
            &self.set == other
        } else {
            debug_assert!(false);
            false
        }
    }

    fn merge(&mut self, other: &ValueSet) -> Result<(), OperationError> {
        if let Some(b) = other.as_uint32_set() {
            mergesets!(self.set, b)
        } else {
            debug_assert!(false);
            Err(OperationError::InvalidValueState)
        }
    }

    fn to_uint32_single(&self) -> Option<u32> {
        if self.set.len() == 1 {
            self.set.iter().copied().take(1).next()
        } else {
            None
        }
    }

    fn as_uint32_set(&self) -> Option<&SmolSet<[u32; 1]>> {
        Some(&self.set)
    }
}

#[cfg(test)]
mod tests {
    use super::ValueSetUint32;
    use crate::prelude::*;

    #[test]
    fn test_valueset_basic() {
        let mut vs = ValueSetUint32::new(0);
        assert_eq!(vs.insert_checked(Value::new_uint32(0)), Ok(false));
        assert_eq!(vs.insert_checked(Value::new_uint32(1)), Ok(true));
        assert_eq!(vs.insert_checked(Value::new_uint32(1)), Ok(false));
    }

    #[test]
    fn test_scim_uint32() {
        let vs: ValueSet = ValueSetUint32::new(69);
        crate::valueset::scim_json_reflexive(&vs, "69");

        // Test that we can parse json values into a valueset.
        crate::valueset::scim_json_put_reflexive::<ValueSetUint32>(&vs, &[])
    }
}
