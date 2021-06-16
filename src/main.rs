use anyhow::anyhow;
use anyhow::Result;
use bee_message::constants::IOTA_SUPPLY;
use bee_message::{output::Output, payload::Payload, prelude::Essence, Message, MessageId};
use std::str::FromStr;

mod message_file;
use message_file::{get_messages_for_outputs, read_messages};
mod snapshot;
use snapshot::{get_snapshot_outputs_and_treasury, OutputData};

const SNAPSHOT_PATH: &str = "full_snapshot.bin";
const MESSAGES_PATH: &str = "snapshot_messages.bin";
const VOTING_MESSAGE_BUILD: &str = "build";
const VOTING_MESSAGE_BURN: &str = "burn";

#[tokio::main]
async fn main() -> Result<()> {
    // if you have a node with the full history you can create the message file yourself
    // get_messages_for_outputs(
    //     Some("http://localhost:14265"),
    //     None,
    //     SNAPSHOT_PATH,
    //     MESSAGES_PATH,
    // )
    // .await?;

    let voting_result = validate_and_count_votes(SNAPSHOT_PATH, MESSAGES_PATH)?;
    println!("{:#?}", voting_result);
    Ok(())
}

#[derive(Debug)]
struct VotingResult {
    iotas_voted_for_build: u64,
    iotas_voted_for_burn: u64,
    iotas_not_voted: u64,
    amount_votes_for_build: usize,
    amount_votes_for_burn: usize,
    amount_not_voted: usize,
}

// Check if messages are the correct ones by comparing the message_id and then count the votes
fn validate_and_count_votes(snapshot_path: &str, messages_path: &str) -> Result<VotingResult> {
    let (outputs, treasury_output_amount) = get_snapshot_outputs_and_treasury(snapshot_path)?;
    let total_output_amount = outputs.len();
    //ignore message ids from genesis snapshot because they don't exist
    let filtered_outputs: Vec<OutputData> = outputs
        .into_iter()
        .filter(|output| {
            output.message_id
                != MessageId::from_str(
                    "0000000000000000000000000000000000000000000000000000000000000000",
                )
                .expect("Couldn't create message id")
        })
        .collect();

    let messages = read_messages(messages_path)?;

    let mut build = Vec::new();
    let mut burn = Vec::new();
    let mut iotas_not_voted = 0;
    // Find out if an output voted for one result
    for output in filtered_outputs {
        let message = messages.get(&output.message_id).unwrap_or_else(|| {
            panic!(
                "Missing message {} for output {}",
                output.message_id, output.output_id
            )
        });
        if let Ok(data) = get_indexation_data(message) {
            if data == VOTING_MESSAGE_BUILD {
                build.push(output);
            } else if data == VOTING_MESSAGE_BURN {
                burn.push(output);
            } else {
                iotas_not_voted += get_output_amount(&output.output);
            }
        } else {
            iotas_not_voted += get_output_amount(&output.output);
        }
    }
    let mut voting_result = VotingResult {
        iotas_voted_for_build: 0,
        iotas_voted_for_burn: 0,
        iotas_not_voted,
        amount_votes_for_build: build.len(),
        amount_votes_for_burn: burn.len(),
        amount_not_voted: total_output_amount - build.len() - burn.len(),
    };
    for output_data in build {
        voting_result.iotas_voted_for_build += get_output_amount(&output_data.output);
    }
    for output_data in burn {
        voting_result.iotas_voted_for_burn += get_output_amount(&output_data.output);
    }

    // validate supply
    assert_eq!(
        voting_result.iotas_voted_for_build
            + voting_result.iotas_voted_for_burn
            + voting_result.iotas_not_voted
            + treasury_output_amount,
        IOTA_SUPPLY
    );
    Ok(voting_result)
}

fn get_output_amount(output: &Output) -> u64 {
    match output {
        Output::Treasury(_) => panic!("Treasury output can't vote"),
        Output::SignatureLockedSingle(output) => output.amount(),
        Output::SignatureLockedDustAllowance(output) => output.amount(),
    }
}

// only returns data from indexation payloads inside of a transaction
fn get_indexation_data(message: &Message) -> Result<String> {
    match message.payload() {
        Some(Payload::Transaction(tx_payload)) => {
            let Essence::Regular(essence) = tx_payload.essence();
            match essence.payload() {
                Some(Payload::Indexation(indexation_payload)) => {
                    let data = String::from_utf8(indexation_payload.data().to_vec())?;
                    Ok(data)
                }
                _ => Err(anyhow!("No indexation payload")),
            }
        }
        _ => Err(anyhow!("No transaction payload")),
    }
}
