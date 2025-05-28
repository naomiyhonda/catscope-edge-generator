use solana_sdk::pubkey::Pubkey;

#[derive(Debug)]
pub enum CatscopeWasmError {
    NotFound(String),     // Variant for "not found" errors with an optional message
    InvalidInput(String), // Variant for invalid input errors with a description
    NetworkError,         // Variant for network-related errors
    InsufficientBuffer,   // the buffer requested is bigger than what is allowed
    InsufficientMemory,   // the buffer requested is bigger than what is allowed
    OutOfRange,           // the index is not in the array
    TimeOut,
    FailedToParse,   // the index is not in the array
    UnknownAccount,  // the index is not in the array
    Unknown(String), // just use a string
    MissingWasmBytes,
    WasmFailure(String),
    GenericError(Box<dyn std::error::Error + Send + Sync>),
    TransactionError(Box<dyn std::error::Error + Send + Sync>),
    DoubleWriting, // cannot write to a Blob once it has been set
    PayloadTooBig(usize),
    SliceWrongSize,
    EmptyPayload,
    NotImplemented,
    MissingHeader,
    MissingSlotGraphNode,
    MissingProgramId,
    InsufficientCpu,
    UnknownMessageType,
    MismatchedHash,
    VersionMismatch,
    PubkeyNodeIdMismatch,
    MissingPubkey(Pubkey),
    MissingEnvironmentalVariable(String),
}

impl std::fmt::Display for CatscopeWasmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CatscopeWasmError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            CatscopeWasmError::InvalidInput(msg) => write!(f, "Invalid Input: {}", msg),
            CatscopeWasmError::WasmFailure(msg) => write!(f, "WASM failure: {}", msg),
            CatscopeWasmError::MissingWasmBytes => write!(f, "missing wasm bytes"),
            CatscopeWasmError::NetworkError => write!(f, "Network Error occurred"),
            CatscopeWasmError::FailedToParse => write!(f, "failed to parse"),
            CatscopeWasmError::EmptyPayload => write!(f, "empty payload"),
            CatscopeWasmError::UnknownAccount => write!(f, "unknown account"),
            CatscopeWasmError::InsufficientBuffer => write!(f, "Requested buffer is too large"),
            CatscopeWasmError::InsufficientMemory => write!(f, "Requested memory is too large"),
            CatscopeWasmError::OutOfRange => write!(f, "index is out of range"),
            CatscopeWasmError::GenericError(e) => write!(f, "generic: {}", e),
            CatscopeWasmError::TransactionError(e) => write!(f, "transaction: {}", e),
            CatscopeWasmError::PayloadTooBig(s) => write!(f, "payload {} is too big", s),
            CatscopeWasmError::Unknown(e) => write!(f, "unknown: {}", e),
            CatscopeWasmError::TimeOut => write!(f, "timed out"),
            CatscopeWasmError::DoubleWriting => {
                write!(f, "cannot write to same blob multiple times")
            }
            CatscopeWasmError::NotImplemented => {
                write!(f, "not implemented yet")
            }
            CatscopeWasmError::InsufficientCpu => write!(f, "insufficient cpu"),
            CatscopeWasmError::MismatchedHash => write!(f, "mismatching hash"),
            CatscopeWasmError::MissingHeader => write!(f, "missing header"),
            CatscopeWasmError::MissingSlotGraphNode => write!(f, "missing slot graph node"),
            CatscopeWasmError::UnknownMessageType => write!(f, "unknown message type"),
            CatscopeWasmError::VersionMismatch => write!(f, "message version mismatch"),
            CatscopeWasmError::MissingProgramId => write!(f, "missing program id"),
            CatscopeWasmError::PubkeyNodeIdMismatch => write!(f, "mismatching program id"),
            CatscopeWasmError::MissingPubkey(e) => write!(f, "missing pubkey {}", e),
            CatscopeWasmError::SliceWrongSize => write!(f, "slice is the wrong size"),
            CatscopeWasmError::MissingEnvironmentalVariable(e) => {
                write!(f, "missing environmental variable {}", e)
            }
        }
    }
}
