#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilitySetMismatch {
    pub required_capability_set_keys: Vec<String>,
    pub producer_capability_set_keys: Vec<String>,
    pub missing_capability_set_keys: Vec<String>,
}

pub const WEBIMAGE_INDEXABLE_CAPABILITY_KEY: &str =
    "Indexable(rank=1,index_type=rich_value_key,element_value_class=rich_value_data)";
pub const WEBIMAGE_SHAPED_CAPABILITY_KEY: &str = "Shaped(extent_class=webimage_kvp_record)";
pub const WEBIMAGE_MATERIALISABLE_CAPABILITY_KEY: &str =
    "Materialisable(target_class=published_fallback_text)";

pub fn webimage_producer_capability_set_keys() -> Vec<String> {
    vec![
        WEBIMAGE_INDEXABLE_CAPABILITY_KEY.to_string(),
        WEBIMAGE_SHAPED_CAPABILITY_KEY.to_string(),
        WEBIMAGE_MATERIALISABLE_CAPABILITY_KEY.to_string(),
    ]
}

pub fn sorted_stable_key_join(keys: &[String]) -> String {
    stable_sorted_deduped(keys).join("|")
}

pub fn ensure_capability_superset(
    required_capability_set_keys: &[String],
    producer_capability_set_keys: &[String],
) -> Result<(), CapabilitySetMismatch> {
    let required = stable_sorted_deduped(required_capability_set_keys);
    let producer = stable_sorted_deduped(producer_capability_set_keys);
    let missing = required
        .iter()
        .filter(|required_key| !producer.contains(required_key))
        .cloned()
        .collect::<Vec<_>>();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(CapabilitySetMismatch {
            required_capability_set_keys: required,
            producer_capability_set_keys: producer,
            missing_capability_set_keys: missing,
        })
    }
}

fn stable_sorted_deduped(keys: &[String]) -> Vec<String> {
    let mut keys = keys.to_vec();
    keys.sort();
    keys.dedup();
    keys
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_superset_accepts_duplicate_and_unsorted_producer_keys() {
        let required = vec![
            WEBIMAGE_MATERIALISABLE_CAPABILITY_KEY.to_string(),
            WEBIMAGE_INDEXABLE_CAPABILITY_KEY.to_string(),
        ];
        let producer = vec![
            WEBIMAGE_INDEXABLE_CAPABILITY_KEY.to_string(),
            WEBIMAGE_INDEXABLE_CAPABILITY_KEY.to_string(),
            WEBIMAGE_SHAPED_CAPABILITY_KEY.to_string(),
            WEBIMAGE_MATERIALISABLE_CAPABILITY_KEY.to_string(),
        ];

        assert_eq!(ensure_capability_superset(&required, &producer), Ok(()));
    }

    #[test]
    fn capability_superset_reports_deterministic_missing_keys() {
        let required = vec![
            WEBIMAGE_SHAPED_CAPABILITY_KEY.to_string(),
            WEBIMAGE_INDEXABLE_CAPABILITY_KEY.to_string(),
        ];
        let producer = vec![WEBIMAGE_SHAPED_CAPABILITY_KEY.to_string()];

        assert_eq!(
            ensure_capability_superset(&required, &producer),
            Err(CapabilitySetMismatch {
                required_capability_set_keys: vec![
                    WEBIMAGE_INDEXABLE_CAPABILITY_KEY.to_string(),
                    WEBIMAGE_SHAPED_CAPABILITY_KEY.to_string(),
                ],
                producer_capability_set_keys: vec![WEBIMAGE_SHAPED_CAPABILITY_KEY.to_string()],
                missing_capability_set_keys: vec![WEBIMAGE_INDEXABLE_CAPABILITY_KEY.to_string()],
            })
        );
    }
}
