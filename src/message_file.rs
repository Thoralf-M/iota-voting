use crate::snapshot::{get_snapshot_outputs_and_treasury, OutputData};
use anyhow::Result;
use bee_common::packable::{Packable, Read, Write};
use bee_message::{Message, MessageId};
use iota_client::Client;
use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{BufReader, BufWriter},
    path::Path,
    str::FromStr,
};

/// Request messages for the outputs from a node and save them to a file
pub async fn get_messages_for_outputs(
    node: Option<&str>,
    permanode: Option<&str>,
    snapshot_path: &str,
    message_filename: &str,
) -> Result<()> {
    let (outputs, _) = get_snapshot_outputs_and_treasury(snapshot_path)?;
    //ignore message ids from genesis snapshot because they don't exist
    let empty_message_id =
        MessageId::from_str("0000000000000000000000000000000000000000000000000000000000000000")
            .expect("Couldn't create message id");
    let filtered_outputs: Vec<OutputData> = outputs
        .into_iter()
        .filter(|output| output.message_id != empty_message_id)
        .collect();

    let mut client_builder = Client::builder();
    if let Some(url) = node {
        client_builder = client_builder.with_node(url)?;
    }
    if let Some(url) = permanode {
        client_builder = client_builder.with_permanode(url, None, None)?;
    }
    let iota_client = client_builder.finish().await?;

    let mut writer = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create(true)
            .open(Path::new(message_filename))?,
    );
    // Save amount of messages
    (filtered_outputs.len() as u64).pack(&mut writer)?;
    for output in filtered_outputs {
        let message = iota_client.get_message().data(&output.message_id).await?;
        message.pack(&mut writer)?;
    }
    writer.flush()?;
    Ok(())
}

/// Read messages from the file
pub fn read_messages(messages_path: &str) -> Result<HashMap<MessageId, Message>> {
    let mut reader = BufReader::new(
        OpenOptions::new()
            .read(true)
            .open(Path::new(messages_path))?,
    );

    let messages_amount: u64 = u64::unpack_inner::<dyn Read, true>(&mut reader)?;

    let mut messages: HashMap<MessageId, Message> = HashMap::new();
    for _ in 0..messages_amount {
        let message = Message::unpack_unchecked(&mut reader)?;
        messages.insert(message.id().0, message);
    }
    Ok(messages)
}
