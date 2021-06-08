/*
Here I will describe the message format

a simple non parted message is described like this

\sea.0 RECIEVER SENDER message_type %{PAYLOAD}

an example exchange wherein JANLILI requests SERVER1/helloworld.txt

\sea.0 SERVER1 JANLILI req %68 65 6c 6c 6f //While it is reccomended to use file path remininiscent addressing, really you can address your document however you want
\sea.0 JANLILI SERVER1 res %68 65 6c 6c 6f 20 77 6f 72 6c 64

however if the message is larger and must be parted into multiple
messages then the message uses sea.1 instead to mark that it is
chunked and follows a different format

\sea.1 RECIEVER SENDER part_count id action
\sea.2 RECIEVER SENDER current_part id %{PAYLOAD PART}

note: id can be whatever the sender chooses, as long as it is within one standard deviation
of the mean bits of entropy of messages in the network (this means you cannot use too much entropy or too little)

the same exchange but parted would look like this

\sea.0 SERVER1 JANLILI req %68 65 6c 6c 6f // JANLILI makes a request for helloworld.txt to server1
\sea.1 JANLILI SERVER1 2 VRBF765RTAK res // SERVER1 responds and chooses to part the message telling JANLILI the number of parts and the id
\sea.2 JANLILI SERVER1 0 VRBF765RTAK %68 65 6c 6c 6f // SERVER1 continues to send part 1/2 (note indexing starts at 0)
\sea.3 JANLILI SERVER1 1 VRBF765RTAK %20 77 6f 72 6c 64 // SERVER1 sends the final part 2/2 of the message

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
\sea.0 SERVER1 JANLILI req %68 65 6c 6c 6f
\sea.0 JANLILI SERVER1 res %68 65 6c 6c 6f 20 77 6f 72 6c 64
\sea.0 SERVER1 JANLILI ty
\sea.0 JANLILI SERVER1 yw

this example shows JANLILI not thanking the server and the server sarcastically responding yw
\sea.0 SERVER1 JANLILI req %68 65 6c 6c 6f
\sea.0 JANLILI SERVER1 res %68 65 6c 6c 6f 20 77 6f 72 6c 64
// ... a minute passes
\sea.0 JANLILI SERVER1 yw

*/

pub enum MessageType {
    sea0 {
        action: String,
        payload: [u8],
    },
    sea1 {
        action: String,
        id: String,
        parts: u32,
    },
    sea2 {
        part: u32,
        id: String,
        payload: [u8],
    },
}

impl MessageType {
    fn prelude(&self) -> String {
        match self {
            Self::sea0 => r"\sea.0",
            Self::sea1 => r"\sea.1",
            Self::sea2 => r"\sea.2",
        }
    }
}

pub struct Message {
    message_type: MessageType,
    sender: String,
    reciever: String,
}

impl Message {
    pub fn from_string(message: String) -> Self {}

    pub fn to_string(&self) -> String {
        match self.message_type {
            MessageType::sea0 => format!(
                "{} {} {} {} %{}",
                self.message_type.prelude(),
                self.reciever,
                self.sender,
                self.message_type.action,
                self.message_type.payload.map(|x| format!("{:x}", x))
            ),
        }
    }

    pub fn make_resource_request(sender: String, reciever: String, resource: String) -> Self {
        Self {
            message_type: MessageType::sea0 {
                action: "req",
                payload: *resource.as_bytes(),
            },
            sender,
            reciever,
        }
    }
}
