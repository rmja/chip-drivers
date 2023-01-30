use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::commands::gsm::PinStatusCode;

impl Serialize for PinStatusCode {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            PinStatusCode::Ready => serializer.serialize_bytes(b"READY"),
            PinStatusCode::SimPin => serializer.serialize_bytes(b"SIM PIN"),
            PinStatusCode::SimPuk => serializer.serialize_bytes(b"SIM PUK"),
            PinStatusCode::PhSimPin => serializer.serialize_bytes(b"PH_SIM PIN"),
            PinStatusCode::PhSimPuk => serializer.serialize_bytes(b"PH_SIM PUK"),
            PinStatusCode::SimPin2 => serializer.serialize_bytes(b"SIM PIN2"),
            PinStatusCode::SimPuk2 => serializer.serialize_bytes(b"SIM PUK2"),
        }
    }
}

impl<'de> Deserialize<'de> for PinStatusCode {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Ready,
            SimPin,
            SimPuk,
            PhSimPin,
            PhSimPuk,
            SimPin2,
            SimPuk2,
        }
        struct FieldVisitor;

        impl<'de> de::Visitor<'de> for FieldVisitor {
            type Value = Field;
            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                core::fmt::Formatter::write_str(formatter, "variant identifier")
            }

            fn visit_bytes<E>(self, value: &[u8]) -> core::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    b"READY" => Ok(Field::Ready),
                    b"SIM PIN" => Ok(Field::SimPin),
                    b"SIM PUK" => Ok(Field::SimPuk),
                    b"PH_SIM PIN" => Ok(Field::PhSimPin),
                    b"PH_SIM PUK" => Ok(Field::PhSimPuk),
                    b"SIM PIN2" => Ok(Field::SimPin2),
                    b"SIM PUK2" => Ok(Field::SimPuk2),

                    _ => {
                        let value =
                            core::str::from_utf8(value).unwrap_or("\u{fffd}\u{fffd}\u{fffd}");
                        Err(de::Error::unknown_variant(value, VARIANTS))
                    }
                }
            }
        }

        impl<'de> Deserialize<'de> for Field {
            #[inline]
            fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                Deserializer::deserialize_identifier(deserializer, FieldVisitor)
            }
        }
        struct Visitor<'de> {
            marker: core::marker::PhantomData<PinStatusCode>,
            lifetime: core::marker::PhantomData<&'de ()>,
        }
        impl<'de> de::Visitor<'de> for Visitor<'de> {
            type Value = PinStatusCode;
            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                core::fmt::Formatter::write_str(formatter, "enum PinStatusCode")
            }

            fn visit_enum<A>(self, data: A) -> core::result::Result<Self::Value, A::Error>
            where
                A: de::EnumAccess<'de>,
            {
                Ok(match de::EnumAccess::variant(data)? {
                    (Field::Ready, _) => PinStatusCode::Ready,
                    (Field::SimPin, _) => PinStatusCode::SimPin,
                    (Field::SimPuk, _) => PinStatusCode::SimPuk,
                    (Field::PhSimPin, _) => PinStatusCode::PhSimPin,
                    (Field::PhSimPuk, _) => PinStatusCode::PhSimPuk,
                    (Field::SimPin2, _) => PinStatusCode::SimPin2,
                    (Field::SimPuk2, _) => PinStatusCode::SimPuk2,
                })
            }
        }
        const VARIANTS: &[&str] = &[
            "Ready", "SimPin", "SimPuk", "PhSimPin", "PhSimPuk", "SimPin2", "SimPuk2",
        ];
        deserializer.deserialize_enum(
            "PinStatusCode",
            VARIANTS,
            Visitor {
                marker: core::marker::PhantomData::<PinStatusCode>,
                lifetime: core::marker::PhantomData,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use atat::serde_at::from_str;

    use crate::commands::gsm::{PinStatus, PinStatusCode};

    #[test]
    fn can_deserialize() {
        let response: PinStatus = from_str(&"+CPIN: READY").unwrap();
        assert_eq!(PinStatusCode::Ready, response.code);
    }
}
