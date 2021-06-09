use std::{convert::TryInto, error::Error, fmt};
use uuid::Uuid;

/*
Here I will describe the message format

a simple non parted message is described like this

\sea.0 RECIEVER SENDER message_type %{PAYLOAD}

an example exchange wherein JANLILI requests SERVER1/helloworld.txt

\sea.0 SERVER1 JANLILI req %/helloworld.txt //While it is reccomended to use file path remininiscent addressing, really you can address your document however you want
\sea.0 JANLILI SERVER1 res %Hello World

however if the message is larger and must be parted into multiple
messages then the message uses sea.1 instead to mark that it is
chunked and follows a different format

\sea.1 RECIEVER SENDER part_count id action
\sea.2 RECIEVER SENDER current_part id %d

note: id can be whatever the sender chooses, as long as it is within one standard deviation
of the mean bits of entropy of messages in the network (this means you cannot use too much entropy or too little)

the same exchange but parted would look like this

\sea.0 SERVER1 JANLILI req %/helloworld.txt // JANLILI makes a request for helloworld.txt to server1
\sea.1 JANLILI SERVER1 2 VRBF765RTAK res // SERVER1 responds and chooses to part the message telling JANLILI the number of parts and the id
\sea.2 JANLILI SERVER1 00000000 VRBF765RTAK %hello // SERVER1 continues to send part 1/2 (note indexing starts at 0)
\sea.2 JANLILI SERVER1 00000001 VRBF765RTAK % world // SERVER1 sends the final part 2/2 of the message

while not required, common manners dictate that the client thanks the server after a successful request, however
if the client deems the service to be poor, the payload is incomplete, or the client is a young punk who doesn't
know manners then this message can be forgone.

\sea.0 RECIEVER SENDER ty

while also not required, common manners dictate that the server should respond to the clients thank you, with a
your welcome, however this can be forgone if the client was especially rude.

\sea.0 RECIEVER SENDER yw

an alternative form is allowed for if the request took less than 50 ms to process on the server side
\sea.0 RECIEVER SENDER np

finally, the server can choose to sarcastically respond with your welcome anyways even though they had not been thanked

here are a few examples of these situations

this example shows a typical polite request
\sea.0 SERVER1 JANLILI req %/helloworld.txt
\sea.0 JANLILI SERVER1 res %Hello World!
\sea.0 SERVER1 JANLILI ty
\sea.0 JANLILI SERVER1 yw

this example shows JANLILI not thanking the server and the server sarcastically responding yw
\sea.0 SERVER1 JANLILI req %/helloworld.txt
\sea.0 JANLILI SERVER1 res %Hello World!
// ... a minute passes
\sea.0 JANLILI SERVER1 yw

*/

const max_message_length: usize = 510;

#[derive(Clone)]
pub enum MessageType {
    Sea0 {
        action: String,
        payload: String,
    },
    Sea1 {
        action: String,
        id: String,
        parts: u32,
    },
    Sea2 {
        part: u32,
        id: String,
        payload: String,
    },
}

impl MessageType {
    fn prelude(&self) -> String {
        match self {
            MessageType::Sea0 { action, payload } => r"\sea.0".to_string(),
            MessageType::Sea1 { action, id, parts } => r"\sea.1".to_string(),
            MessageType::Sea2 { part, id, payload } => r"\sea.2".to_string(),
        }
    }
}

pub struct Message {
    message_type: MessageType,
    sender: String,
    reciever: String,
}

pub enum MessageParseError {
    EmptyMessageContent,
    UnknownSeaDotProtocol,
    IncompleteHeader,
    ParseIntError,
}

impl Message {
    pub fn from_string(message: String) -> Result<Self, MessageParseError> {
        let mut header;
        let mut payload = "".to_string();

        let index = message.find("%");

        match index {
            Some(index) => {
                let tuple = message.split_at(index);
                header = tuple.0.to_string();
                payload = tuple.1.to_string();
            }
            None => header = message,
        }

        payload = payload.strip_prefix("%").unwrap_or(&payload).to_string();

        let header = header.split_whitespace().collect::<Vec<_>>();

        let sender = header
            .get(2)
            .ok_or(MessageParseError::IncompleteHeader)?
            .to_string();
        let reciever = header
            .get(1)
            .ok_or(MessageParseError::IncompleteHeader)?
            .to_string();

        match *header
            .get(0)
            .ok_or(MessageParseError::EmptyMessageContent)?
        {
            r"\sea.0" => Ok(Message {
                message_type: MessageType::Sea0 {
                    action: header
                        .get(3)
                        .ok_or(MessageParseError::IncompleteHeader)?
                        .to_string(),
                    payload,
                },
                sender,
                reciever,
            }),
            r"\sea.1" => Ok(Message {
                message_type: MessageType::Sea1 {
                    action: header
                        .get(5)
                        .ok_or(MessageParseError::IncompleteHeader)?
                        .to_string(),
                    id: header
                        .get(4)
                        .ok_or(MessageParseError::IncompleteHeader)?
                        .to_string(),
                    parts: header
                        .get(3)
                        .ok_or(MessageParseError::IncompleteHeader)?
                        .to_string()
                        .parse::<u32>()
                        .ok()
                        .ok_or(MessageParseError::ParseIntError)?,
                },
                sender,
                reciever,
            }),
            r"\sea.2" => Ok(Message {
                message_type: MessageType::Sea2 {
                    part: header
                        .get(3)
                        .ok_or(MessageParseError::IncompleteHeader)?
                        .to_string()
                        .parse::<u32>()
                        .ok()
                        .ok_or(MessageParseError::ParseIntError)?,
                    id: header
                        .get(4)
                        .ok_or(MessageParseError::IncompleteHeader)?
                        .to_string(),
                    payload,
                },
                sender,
                reciever,
            }),
            _ => Err(MessageParseError::UnknownSeaDotProtocol),
        }
    }

    pub fn make_resource_request<S: AsRef<str>>(sender: S, reciever: S, resource: S) -> Self {
        let (sender, reciever, resource) = (sender.as_ref().to_string(), reciever.as_ref().to_string(), resource.as_ref().to_string());
        Self {
            message_type: MessageType::Sea0 {
                action: "req".to_string(),
                payload: resource,
            },
            sender,
            reciever,
        }
    }

    pub fn make_resource_response<S: AsRef<str>>(sender: S, reciever: S, payload: S) -> Vec<Self> {
        let (sender, reciever, payload) = (sender.as_ref(), reciever.as_ref(), payload.as_ref());
        // Why didn't I just use json for this ugh
        let message = Self {
            message_type: MessageType::Sea0 {
                action: "res".to_string(),
                payload: payload.to_string(), // First try to send Sea0 message
            },
            sender: sender.to_string(),
            reciever: reciever.to_string(),
        };

        if message.to_string().len() < max_message_length {
            return vec![message];
        }

        let uuid = Uuid::new_v4();
        let id = format!("{}", uuid);

        let message_part_header_length = Self {
            message_type: MessageType::Sea2 {
                part: 0,
                id: id.to_string(),
                payload: "".to_string(),
            },
            sender: sender.to_string(),
            reciever: reciever.to_string(),
        }
        .to_string()
        .len();

        let payload_part_length = max_message_length - message_part_header_length;

        let payload_parts = (payload.len() as f32 / payload_part_length as f32).ceil() as usize;

        let message_parted_begin = Self {
            message_type: MessageType::Sea1 {
                action: "res".to_string(),
                id: id.to_string(),
                parts: payload_parts as u32,
            },
            sender: sender.to_string(),
            reciever: reciever.to_string(),
        };

        let mut message_parts = vec![message_parted_begin];

        let mut payload_remaining = payload.to_string();

        for i in 0..payload_parts {
            let split = payload_remaining.split_at(std::cmp::min(payload_part_length, payload_remaining.len()));
            let payload_part = split.0.to_string();

            payload_remaining = split.1.to_string();

            message_parts.push(Self {
                message_type: MessageType::Sea2 {
                    part: i as u32,
                    id: id.to_string(),
                    payload: payload_part,
                },
                sender: sender.to_string(),
                reciever: reciever.to_string(),
            })
        }

        message_parts
    }

    pub fn reciever(&self) -> String {
        self.reciever.to_string()
    }

    pub fn sender(&self) -> String {
        self.sender.to_string()
    }

    pub fn message_type(&self) -> MessageType {
        self.message_type.clone()
    }
}

impl fmt::Display for  Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.message_type {
            MessageType::Sea0 { action, payload } => write!(f, 
                "{} {} {} {} %{}",
                self.message_type.prelude(),
                self.reciever,
                self.sender,
                action,
                payload
            ),
            MessageType::Sea1 { action, id, parts } => write!(f, 
                "{} {} {} {} {} {}",
                self.message_type.prelude(),
                self.reciever,
                self.sender,
                parts,
                id,
                action
            ),
            MessageType::Sea2 { part, id, payload } => write!(f, 
                "{} {} {} {:08} {} %{}",
                self.message_type.prelude(),
                self.reciever,
                self.sender,
                part,
                id,
                payload
            ),
        }
    }
}