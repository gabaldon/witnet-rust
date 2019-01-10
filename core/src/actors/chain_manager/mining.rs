use actix::{ActorFuture, Context, ContextFutureSpawner, Handler, System, WrapFuture};

use ansi_term::Color::Yellow;

use super::ChainManager;
use crate::actors::chain_manager::messages::GetHighestCheckpointBeacon;
use crate::actors::epoch_manager::messages::EpochNotification;

use witnet_crypto::hash::calculate_sha256;
use witnet_data_structures::chain::{Hash, LeadershipProof, Secp256k1Signature, Signature};
use witnet_storage::storage::Storable;

use crate::actors::chain_manager::messages::AddNewBlock;
use crate::actors::chain_manager::messages::BuildBlock;
use crate::actors::reputation_manager::messages::ValidatePoE;
use crate::actors::reputation_manager::ReputationManager;
use log::{debug, error, info};

/// Payload for the notification for all epochs
#[derive(Clone, Debug)]
pub struct MiningNotification;

/// Handler for EpochNotification<MiningNotification>
impl Handler<EpochNotification<MiningNotification>> for ChainManager {
    type Result = ();

    fn handle(&mut self, msg: EpochNotification<MiningNotification>, ctx: &mut Context<Self>) {
        debug!("Periodic epoch notification received {:?}", msg.checkpoint);

        // Check eligibility
        // S(H(beacon))
        let mut beacon = match self.handle(GetHighestCheckpointBeacon, ctx) {
            Ok(b) => b,
            _ => return,
        };

        if beacon.checkpoint > msg.checkpoint {
            // We got a block from the future
            error!(
                "The current highest checkpoint beacon is from the future ({:?} > {:?})",
                beacon.checkpoint, msg.checkpoint
            );
            return;
        }
        if beacon.checkpoint == msg.checkpoint {
            // For some reason we already got a valid block for this epoch
            // TODO: Check eligibility anyway?
        }
        // The highest checkpoint beacon should contain the current epoch
        beacon.checkpoint = msg.checkpoint;
        let beacon_hash = Hash::from(calculate_sha256(&beacon.to_bytes().unwrap()));
        let private_key = 1;

        // TODO: send Sign message to CryptoManager
        let sign = |x, _k| match x {
            Hash::SHA256(mut x) => {
                // Add some randomness to the signature
                x[0] = self.random as u8;
                x
            }
        };
        let signed_beacon_hash = sign(beacon_hash, private_key);
        // Currently, every hash is valid
        // Fake signature which will be accepted anyway
        let signature = Signature::Secp256k1(Secp256k1Signature {
            r: signed_beacon_hash,
            s: signed_beacon_hash,
            v: 0,
        });
        let leadership_proof = LeadershipProof {
            block_sig: Some(signature),
            influence: 0,
        };

        // Send ValidatePoE message to ReputationManager
        let reputation_manager_addr = System::current().registry().get::<ReputationManager>();
        reputation_manager_addr
            .send(ValidatePoE {
                beacon,
                proof: leadership_proof,
            })
            .into_actor(self)
            .drop_err()
            .and_then(move |eligible, act, ctx| {
                if eligible {
                    info!(
                        "{} Discovered eligibility for mining a block for epoch #{}",
                        Yellow.bold().paint("[Mining]"),
                        Yellow.bold().paint(beacon.checkpoint.to_string())
                    );
                    // Send proof of eligibility to chain manager,
                    // which will construct and broadcast the block

                    // Build the block using the supplied beacon and eligibility proof
                    let block = act.build_block(&BuildBlock {
                        beacon,
                        leadership_proof,
                    });

                    // Send AddNewBlock message to self
                    // This will run all the validations again
                    act.handle(AddNewBlock { block }, ctx);
                }
                actix::fut::ok(())
            })
            .wait(ctx);
    }
}