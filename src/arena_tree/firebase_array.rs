use serde::{de::{MapAccess, SeqAccess, Visitor}, Deserializer, Serialize, Serializer};
use std::fmt;

pub fn serialize<S, const N: usize>(
    arr: &[Option<usize>; N],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    arr.serialize(serializer)
}

pub fn deserialize<'de, D, const N: usize>(deserializer: D) -> Result<[Option<usize>; N], D::Error>
where
    D: Deserializer<'de>,
{
    struct SparseArrayVisitor<const N: usize>;

    impl<'de, const N: usize> Visitor<'de> for SparseArrayVisitor<N> {
        type Value = [Option<usize>; N];

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a JSON array or a sparse JSON object representing an array")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut result = [None; N];
            let mut idx = 0;

            while let Some(elem) = seq.next_element::<Option<usize>>()? {
                if idx < N {
                    result[idx] = elem;
                }
                idx += 1;
            }

            Ok(result)
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut result = [None; N];

            while let Some((key_str, val)) = map.next_entry::<String, usize>()? {
                if let Ok(idx) = key_str.parse::<usize>() {
                    if idx < N {
                        result[idx] = Some(val);
                    }
                }
            }

            Ok(result)
        }
    }

    deserializer.deserialize_any(SparseArrayVisitor::<N>)
}