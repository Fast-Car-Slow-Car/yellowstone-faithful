use {solana_pubkey::Pubkey, solana_sha256_hasher::hashv, solana_signature::Signature};

const CORRELATION_VERSION: &str = "corr_v1";

pub fn build_correlation_key(
    slot: u64,
    tx_index: Option<u64>,
    tx_signature: Option<&str>,
    kind: &str,
) -> String {
    match (tx_index, tx_signature) {
        (Some(tx_index), Some(tx_signature)) => {
            format!("{CORRELATION_VERSION}:{slot}:{tx_index}:{tx_signature}")
        }
        _ => format!("{CORRELATION_VERSION}:slot:{slot}:{kind}"),
    }
}

pub fn correlation_id_from_key(key: &str) -> u64 {
    let hash = hashv(&[key.as_bytes()]).to_bytes();
    u64::from_be_bytes([
        hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6], hash[7],
    ])
}

pub fn correlation_id_for_transaction(slot: u64, tx_index: u64, tx_signature: &Signature) -> u64 {
    let signature = tx_signature.to_string();
    let key = build_correlation_key(
        slot,
        Some(tx_index),
        Some(signature.as_str()),
        "transaction",
    );
    correlation_id_from_key(&key)
}

pub fn correlation_id_for_kind(slot: u64, kind: &str) -> u64 {
    let key = build_correlation_key(slot, None, None, kind);
    correlation_id_from_key(&key)
}

pub fn correlation_id_for_slot(slot: u64, parent: Option<u64>, status: &str) -> u64 {
    let parent = parent
        .map(|parent| parent.to_string())
        .unwrap_or_else(|| "none".to_owned());
    correlation_id_for_kind(slot, &format!("slot:{status}:parent:{parent}"))
}

pub fn correlation_id_for_account(
    slot: u64,
    pubkey: &Pubkey,
    write_version: u64,
    txn_signature: Option<&Signature>,
) -> u64 {
    let signature = txn_signature
        .map(ToString::to_string)
        .unwrap_or_else(|| "none".to_owned());
    correlation_id_for_kind(
        slot,
        &format!("account:{pubkey}:write:{write_version}:sig:{signature}"),
    )
}

pub fn correlation_id_for_entry(slot: u64, index: usize, starting_transaction_index: u64) -> u64 {
    correlation_id_for_kind(
        slot,
        &format!("entry:index:{index}:start_tx:{starting_transaction_index}"),
    )
}

pub fn correlation_id_for_block_meta(slot: u64, parent_slot: u64, blockhash: &str) -> u64 {
    correlation_id_for_kind(
        slot,
        &format!("block_meta:parent:{parent_slot}:blockhash:{blockhash}"),
    )
}

pub fn correlation_id_for_block(
    slot: u64,
    parent_slot: u64,
    blockhash: &str,
    tx_count: usize,
    account_count: usize,
    entries_count: u64,
) -> u64 {
    correlation_id_for_kind(
        slot,
        &format!(
            "block:parent:{parent_slot}:blockhash:{blockhash}:tx:{tx_count}:accounts:{account_count}:entries:{entries_count}"
        ),
    )
}

#[cfg(test)]
mod tests {
    use {super::*, std::str::FromStr};

    #[test]
    fn test_build_correlation_key_transaction() {
        let key = build_correlation_key(
            42,
            Some(7),
            Some("4V36qYhukXcLFuvhZaudSoJpPaFNB7d5RqYKjL2xiSKrxaBfEajqqL4X6viZkEvHJ8XcTJsqVjZxFegxhN7EC9V5"),
            "transaction",
        );
        assert_eq!(
            key,
            "corr_v1:42:7:4V36qYhukXcLFuvhZaudSoJpPaFNB7d5RqYKjL2xiSKrxaBfEajqqL4X6viZkEvHJ8XcTJsqVjZxFegxhN7EC9V5"
        );
    }

    #[test]
    fn test_build_correlation_key_non_transaction_fallback() {
        let key = build_correlation_key(42, None, None, "slot");
        assert_eq!(key, "corr_v1:slot:42:slot");
    }

    #[test]
    fn test_correlation_id_is_deterministic_for_same_transaction() {
        let signature = Signature::from_str(
            "4V36qYhukXcLFuvhZaudSoJpPaFNB7d5RqYKjL2xiSKrxaBfEajqqL4X6viZkEvHJ8XcTJsqVjZxFegxhN7EC9V5",
        )
        .unwrap();

        let one = correlation_id_for_transaction(99, 3, &signature);
        let two = correlation_id_for_transaction(99, 3, &signature);
        assert_eq!(one, two);
    }

    #[test]
    fn test_transaction_and_fallback_ids_are_different() {
        let signature = Signature::from_str(
            "4V36qYhukXcLFuvhZaudSoJpPaFNB7d5RqYKjL2xiSKrxaBfEajqqL4X6viZkEvHJ8XcTJsqVjZxFegxhN7EC9V5",
        )
        .unwrap();

        let tx = correlation_id_for_transaction(123, 0, &signature);
        let slot = correlation_id_for_kind(123, "slot");
        assert_ne!(tx, slot);
    }

    #[test]
    fn test_account_correlation_changes_with_write_version() {
        let pubkey = Pubkey::new_unique();
        let one = correlation_id_for_account(500, &pubkey, 1, None);
        let two = correlation_id_for_account(500, &pubkey, 2, None);
        assert_ne!(one, two);
    }

    #[test]
    fn test_entry_correlation_changes_with_index() {
        let one = correlation_id_for_entry(700, 1, 10);
        let two = correlation_id_for_entry(700, 2, 10);
        assert_ne!(one, two);
    }
}
