use chrono::{DateTime, Utc};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Timestamp(DateTime<Utc>);

impl From<DateTime<Utc>> for Timestamp {
    fn from(value: DateTime<Utc>) -> Self {
        Self(value)
    }
}

impl From<Timestamp> for DateTime<Utc> {
    fn from(value: Timestamp) -> Self {
        value.0
    }
}

impl<C> bincode::de::Decode<C> for Timestamp {
    fn decode<D: bincode::de::Decoder<Context = C>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let timestamp_s: i64 = u32::decode(decoder)?.into();

        DateTime::from_timestamp(timestamp_s, 0)
            .map(Timestamp)
            .ok_or(bincode::error::DecodeError::Other("invalid epoch second"))
    }
}

impl<'de, C> bincode::de::BorrowDecode<'de, C> for Timestamp {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = C>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        bincode::Decode::decode(decoder)
    }
}

impl bincode::enc::Encode for Timestamp {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        let timestamp_s: u32 = self
            .0
            .timestamp()
            .try_into()
            .map_err(|_| bincode::error::EncodeError::Other("invalid epoch second"))?;

        bincode::enc::Encode::encode(&timestamp_s, encoder)
    }
}
