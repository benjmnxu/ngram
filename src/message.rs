/// A request from the client to the server
#[derive(Debug, PartialEq)]
pub enum Request {
    /// Add the document `doc` to the archive
    Publish { doc: String },
    /// Search for the word `word` in the archive
    Search { word: String },
    /// Retrieve the document with the index `id` from the archive
    Retrieve { id: usize },
}

impl Request {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut payload = match self {
            Self::Publish { doc } => {
                let mut bytes = vec![0u8]; // 0 represents "Publish"
                bytes.extend(doc.as_bytes());
                bytes
            }
            Self::Search { word } => {
                let mut bytes = vec![1u8]; // 1 represents "Search"
                bytes.extend(word.as_bytes());
                bytes
            }
            Self::Retrieve { id } => {
                let mut bytes = vec![2u8]; // 2 represents "Retrieve"
                bytes.extend(&id.to_be_bytes());
                bytes
            }
        };

        // Prepend the length of the payload as a 4-byte big-endian integer
        let mut message = (payload.len() as u32).to_be_bytes().to_vec();
        message.append(&mut payload);
        message
    }

    pub fn from_bytes<R: std::io::Read>(mut reader: R) -> Option<Self> {
        // Read the length (4 bytes)
        let mut length_bytes = [0u8; 4];
        if reader.read_exact(&mut length_bytes).is_err() {
            return None;
        }
        let length = u32::from_be_bytes(length_bytes) as usize;

        // Read the remaining payload based on the length
        let mut payload = vec![0u8; length];
        if reader.read_exact(&mut payload).is_err() {
            return None;
        }

        // Extract the variant identifier
        let variant = payload[0];
        match variant {
            0 => {
                let doc = String::from_utf8(payload[1..].to_vec()).ok()?;
                Some(Self::Publish { doc })
            }
            1 => {
                let word = String::from_utf8(payload[1..].to_vec()).ok()?;
                Some(Self::Search { word })
            }
            2 => {
                if payload.len() >= 1 + std::mem::size_of::<usize>() {
                    let mut id_bytes = [0u8; std::mem::size_of::<usize>()];
                    id_bytes.copy_from_slice(&payload[1..1 + std::mem::size_of::<usize>()]);
                    Some(Self::Retrieve {
                        id: usize::from_be_bytes(id_bytes),
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}


/// A response from the server to the client
#[derive(Debug, PartialEq)]
pub enum Response {
    /// The document was successfully added to the archive with the given index
    PublishSuccess(usize),
    /// The search for the word was successful, and the indices of the documents containing the
    /// word are returned
    SearchSuccess(Vec<usize>),
    /// The retrieval of the document was successful, and the document is returned
    RetrieveSuccess(String),
    /// The request failed
    Failure,
}
impl Response {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut payload = match self {
            Self::PublishSuccess(index) => {
                let mut bytes = vec![0u8]; // 0 represents "PublishSuccess"
                bytes.extend_from_slice(&index.to_be_bytes());
                bytes
            }
            Self::SearchSuccess(indices) => {
                let mut bytes = vec![1u8]; // 1 represents "SearchSuccess"
                for index in indices {
                    bytes.extend(index.to_be_bytes());
                }
                bytes
            }
            Self::RetrieveSuccess(doc) => {
                let mut bytes = vec![2u8]; // 2 represents "RetrieveSuccess"
                bytes.extend_from_slice(doc.as_bytes());
                bytes
            }
            Self::Failure => vec![3u8], // 3 represents "Failure"
        };

        let mut message = (payload.len() as u32).to_be_bytes().to_vec();
        message.append(&mut payload);
        message
    }

    pub fn from_bytes<R: std::io::Read>(mut reader: R) -> Option<Self> {
        let mut length_bytes = [0u8; 4];
        if reader.read_exact(&mut length_bytes).is_err() {
            return None;
        }
        let length = u32::from_be_bytes(length_bytes) as usize;

        let mut payload = vec![0u8; length];
        if reader.read_exact(&mut payload).is_err() {
            return None;
        }

        let variant = payload[0];
        match variant {
            0 => {
                if payload.len() >= 1 + std::mem::size_of::<usize>() {
                    let mut index_bytes = [0u8; std::mem::size_of::<usize>()];
                    index_bytes.copy_from_slice(&payload[1..1 + std::mem::size_of::<usize>()]);
                    let index = usize::from_be_bytes(index_bytes);
                    Some(Self::PublishSuccess(index))
                } else {
                    None
                }
            }
            1 => {
                let mut indices = Vec::new();
                let mut start = 1;
                while start + 8 <= payload.len() {
                    let mut index_bytes = [0u8; 8];
                    index_bytes.copy_from_slice(&payload[start..start + 8]);
                    indices.push(usize::from_be_bytes(index_bytes));
                    start += 8;
                }
                Some(Self::SearchSuccess(indices))
            }
            2 => {
                let doc = String::from_utf8(payload[1..].to_vec()).ok()?;
                Some(Self::RetrieveSuccess(doc))
            }
            3 => Some(Self::Failure),
            _ => None,
        }
    }
}
