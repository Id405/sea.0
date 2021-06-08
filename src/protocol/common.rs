use std::{convert::TryInto, error::Error};

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
\sea.2 JANLILI SERVER1 0 VRBF765RTAK %hello // SERVER1 continues to send part 1/2 (note indexing starts at 0)
\sea.2 JANLILI SERVER1 1 VRBF765RTAK % world // SERVER1 sends the final part 2/2 of the message

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

    pub fn to_string(&self) -> String {
        match &self.message_type {
            MessageType::Sea0 { action, payload } => format!(
                "{} {} {} {} %{}",
                self.message_type.prelude(),
                self.reciever,
                self.sender,
                action,
                payload
            ),
            MessageType::Sea1 { action, id, parts } => todo!(),
            MessageType::Sea2 { part, id, payload } => todo!(),
        }
    }

    pub fn make_resource_request(sender: String, reciever: String, resource: String) -> Self {
        Self {
            message_type: MessageType::Sea0 {
                action: "req".to_string(),
                payload: resource,
            },
            sender,
            reciever,
        }
    }
}
