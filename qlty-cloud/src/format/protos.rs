use super::Formatter;
use prost::{bytes::BytesMut, Message};
use std::io::Write;

#[derive(Debug)]
pub struct ProtosFormatter<T>
where
    T: IntoIterator,
    T::Item: Message,
{
    records: T,
}

impl<T> ProtosFormatter<T>
where
    T: IntoIterator + Clone + 'static,
    T::Item: Message,
{
    pub fn new(records: T) -> Box<dyn Formatter> {
        Box::new(Self {
            records: records.clone(),
        })
    }
}

impl<T> Formatter for ProtosFormatter<T>
where
    T: IntoIterator + Clone,
    T::Item: Message,
{
    fn write_to(&self, writer: &mut dyn Write) -> anyhow::Result<()> {
        let mut buffer = BytesMut::new();

        for record in self.records.clone().into_iter() {
            record.encode_length_delimited(&mut buffer).unwrap();
        }

        writer.write_all(&buffer)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ProtoFormatter<T: Message> {
    record: T,
}

impl<T: Message + 'static> ProtoFormatter<T> {
    pub fn new(record: T) -> Box<dyn Formatter> {
        Box::new(Self { record })
    }
}

impl<T: Message> Formatter for ProtoFormatter<T> {
    fn write_to(&self, writer: &mut dyn Write) -> anyhow::Result<()> {
        let mut buffer = BytesMut::new();
        self.record.encode(&mut buffer)?;
        writer.write_all(&buffer)?;
        Ok(())
    }
}