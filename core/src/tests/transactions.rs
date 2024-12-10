use crate::imports::*;
use kaspa_wallet_core::tx::mass::MassCalculator;

#[test]
fn test_generic_transaction_mass() {
    use kaspa_consensus_core::tx::*;

    let script_public_key = ScriptPublicKey::new(
        0,
        smallvec::smallvec![
            0x76, 0xa9, 0x21, 0x03, 0x2f, 0x7e, 0x43, 0x0a, 0xa4, 0xc9, 0xd1, 0x59, 0x43, 0x7e,
            0x84, 0xb9, 0x75, 0xdc, 0x76, 0xd9, 0x00, 0x3b, 0xf0, 0x92, 0x2c, 0xf3, 0xaa, 0x45,
            0x28, 0x46, 0x4b, 0xab, 0x78, 0x0d, 0xba, 0x5e
        ],
    );
    let tx = Transaction::new(
        0,
        vec![
            TransactionInput {
                previous_outpoint: TransactionOutpoint {
                    transaction_id: TransactionId::from_slice(&[
                        0x16, 0x5e, 0x38, 0xe8, 0xb3, 0x91, 0x45, 0x95, 0xd9, 0xc6, 0x41, 0xf3,
                        0xb8, 0xee, 0xc2, 0xf3, 0x46, 0x11, 0x89, 0x6b, 0x82, 0x1a, 0x68, 0x3b,
                        0x7a, 0x4e, 0xde, 0xfe, 0x2c, 0x00, 0x00, 0x00,
                    ]),
                    index: 0xffffffff,
                },
                signature_script: vec![1; 32],
                sequence: u64::MAX,
                sig_op_count: 0,
            },
            TransactionInput {
                previous_outpoint: TransactionOutpoint {
                    transaction_id: TransactionId::from_slice(&[
                        0x4b, 0xb0, 0x75, 0x35, 0xdf, 0xd5, 0x8e, 0x0b, 0x3c, 0xd6, 0x4f, 0xd7,
                        0x15, 0x52, 0x80, 0x87, 0x2a, 0x04, 0x71, 0xbc, 0xf8, 0x30, 0x95, 0x52,
                        0x6a, 0xce, 0x0e, 0x38, 0xc6, 0x00, 0x00, 0x00,
                    ]),
                    index: 0xffffffff,
                },
                signature_script: vec![1; 32],
                sequence: u64::MAX,
                sig_op_count: 0,
            },
        ],
        vec![
            TransactionOutput {
                value: 300,
                script_public_key: script_public_key.clone(),
            },
            TransactionOutput {
                value: 300,
                script_public_key,
            },
        ],
        0,
        kaspa_consensus_core::subnets::SUBNETWORK_ID_COINBASE,
        0,
        vec![9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    );

    for network in Network::iter() {
        let calc = MassCalculator::new(&network.into());
        let mass = calc.calc_compute_mass_for_unsigned_consensus_transaction(&tx, 1);
        println!("compute transaction mass for {} is {}", network, mass);
    }
}
