/**
* Definition of the legal "magic" values used in a packet.
* See section 3.1 Magic byte
*/
#[derive(Debug)]
pub struct Magic(pub u8);
pub const PROTOCOL_BINARY_REQ : Magic = Magic(0x80);
pub const PROTOCOL_BINARY_RES : Magic = Magic(0x81);

/**
* Definition of the valid response status numbers.
* See section 3.2 Response Status
*/
#[derive(Debug)]
pub struct ResponseStatus(pub u16);
pub const PROTOCOL_BINARY_RESPONSE_SUCCESS : ResponseStatus = ResponseStatus(0x00);
pub const PROTOCOL_BINARY_RESPONSE_KEY_ENOENT : ResponseStatus = ResponseStatus(0x01);
pub const PROTOCOL_BINARY_RESPONSE_KEY_EEXISTS : ResponseStatus = ResponseStatus(0x02);
pub const PROTOCOL_BINARY_RESPONSE_E2BIG : ResponseStatus = ResponseStatus(0x03);
pub const PROTOCOL_BINARY_RESPONSE_EINVAL : ResponseStatus = ResponseStatus(0x04);
pub const PROTOCOL_BINARY_RESPONSE_NOT_STORED : ResponseStatus = ResponseStatus(0x05);
pub const PROTOCOL_BINARY_RESPONSE_DELTA_BADVAL : ResponseStatus = ResponseStatus(0x06);
pub const PROTOCOL_BINARY_RESPONSE_NOT_MY_VBUCKET : ResponseStatus = ResponseStatus(0x07);
pub const PROTOCOL_BINARY_RESPONSE_AUTH_ERROR : ResponseStatus = ResponseStatus(0x20);
pub const PROTOCOL_BINARY_RESPONSE_AUTH_CONTINUE : ResponseStatus = ResponseStatus(0x21);
pub const PROTOCOL_BINARY_RESPONSE_UNKNOWN_COMMAND : ResponseStatus = ResponseStatus(0x81);
pub const PROTOCOL_BINARY_RESPONSE_ENOMEM : ResponseStatus = ResponseStatus(0x82);
pub const PROTOCOL_BINARY_RESPONSE_NOT_SUPPORTED : ResponseStatus = ResponseStatus(0x83);
pub const PROTOCOL_BINARY_RESPONSE_EINTERNAL : ResponseStatus = ResponseStatus(0x84);
pub const PROTOCOL_BINARY_RESPONSE_EBUSY : ResponseStatus = ResponseStatus(0x85);
pub const PROTOCOL_BINARY_RESPONSE_ETMPFAIL : ResponseStatus = ResponseStatus(0x86);

/**
* Defintion of the different command opcodes.
* See section 3.3 Command Opcodes
*/
#[derive(Debug)]
pub struct Opcode(pub u8);
pub const PROTOCOL_BINARY_CMD_GET : Opcode = Opcode(0x00);
pub const PROTOCOL_BINARY_CMD_SET : Opcode = Opcode(0x01);
pub const PROTOCOL_BINARY_CMD_ADD : Opcode = Opcode(0x02);
pub const PROTOCOL_BINARY_CMD_REPLACE : Opcode = Opcode(0x03);
pub const PROTOCOL_BINARY_CMD_DELETE : Opcode = Opcode(0x04);
pub const PROTOCOL_BINARY_CMD_INCREMENT : Opcode = Opcode(0x05);
pub const PROTOCOL_BINARY_CMD_DECREMENT : Opcode = Opcode(0x06);
pub const PROTOCOL_BINARY_CMD_QUIT : Opcode = Opcode(0x07);
pub const PROTOCOL_BINARY_CMD_FLUSH : Opcode = Opcode(0x08);
pub const PROTOCOL_BINARY_CMD_GETQ : Opcode = Opcode(0x09);
pub const PROTOCOL_BINARY_CMD_NOOP : Opcode = Opcode(0x0a);
pub const PROTOCOL_BINARY_CMD_VERSION : Opcode = Opcode(0x0b);
pub const PROTOCOL_BINARY_CMD_GETK : Opcode = Opcode(0x0c);
pub const PROTOCOL_BINARY_CMD_GETKQ : Opcode = Opcode(0x0d);
pub const PROTOCOL_BINARY_CMD_APPEND : Opcode = Opcode(0x0e);
pub const PROTOCOL_BINARY_CMD_PREPEND : Opcode = Opcode(0x0f);
pub const PROTOCOL_BINARY_CMD_STAT : Opcode = Opcode(0x10);
pub const PROTOCOL_BINARY_CMD_SETQ : Opcode = Opcode(0x11);
pub const PROTOCOL_BINARY_CMD_ADDQ : Opcode = Opcode(0x12);
pub const PROTOCOL_BINARY_CMD_REPLACEQ : Opcode = Opcode(0x13);
pub const PROTOCOL_BINARY_CMD_DELETEQ : Opcode = Opcode(0x14);
pub const PROTOCOL_BINARY_CMD_INCREMENTQ : Opcode = Opcode(0x15);
pub const PROTOCOL_BINARY_CMD_DECREMENTQ : Opcode = Opcode(0x16);
pub const PROTOCOL_BINARY_CMD_QUITQ : Opcode = Opcode(0x17);
pub const PROTOCOL_BINARY_CMD_FLUSHQ : Opcode = Opcode(0x18);
pub const PROTOCOL_BINARY_CMD_APPENDQ : Opcode = Opcode(0x19);
pub const PROTOCOL_BINARY_CMD_PREPENDQ : Opcode = Opcode(0x1a);
pub const PROTOCOL_BINARY_CMD_VERBOSITY : Opcode = Opcode(0x1b);
pub const PROTOCOL_BINARY_CMD_TOUCH : Opcode = Opcode(0x1c);
pub const PROTOCOL_BINARY_CMD_GAT : Opcode = Opcode(0x1d);
pub const PROTOCOL_BINARY_CMD_GATQ : Opcode = Opcode(0x1e);
pub const PROTOCOL_BINARY_CMD_GATK : Opcode = Opcode(0x23);
pub const PROTOCOL_BINARY_CMD_GATKQ : Opcode = Opcode(0x24);

pub const PROTOCOL_BINARY_CMD_SASL_LIST_MECHS : Opcode = Opcode(0x20);
pub const PROTOCOL_BINARY_CMD_SASL_AUTH : Opcode = Opcode(0x21);
pub const PROTOCOL_BINARY_CMD_SASL_STEP : Opcode = Opcode(0x22);

/* These commands are used for range operations and exist within
* this header for use in other projects.  Range operations are
* not expected to be implemented in the memcached server itself.
*/
pub const PROTOCOL_BINARY_CMD_RGET      : Opcode = Opcode(0x30);
pub const PROTOCOL_BINARY_CMD_RSET      : Opcode = Opcode(0x31);
pub const PROTOCOL_BINARY_CMD_RSETQ     : Opcode = Opcode(0x32);
pub const PROTOCOL_BINARY_CMD_RAPPEND   : Opcode = Opcode(0x33);
pub const PROTOCOL_BINARY_CMD_RAPPENDQ  : Opcode = Opcode(0x34);
pub const PROTOCOL_BINARY_CMD_RPREPEND  : Opcode = Opcode(0x35);
pub const PROTOCOL_BINARY_CMD_RPREPENDQ : Opcode = Opcode(0x36);
pub const PROTOCOL_BINARY_CMD_RDELETE   : Opcode = Opcode(0x37);
pub const PROTOCOL_BINARY_CMD_RDELETEQ  : Opcode = Opcode(0x38);
pub const PROTOCOL_BINARY_CMD_RINCR     : Opcode = Opcode(0x39);
pub const PROTOCOL_BINARY_CMD_RINCRQ    : Opcode = Opcode(0x3a);
pub const PROTOCOL_BINARY_CMD_RDECR     : Opcode = Opcode(0x3b);
pub const PROTOCOL_BINARY_CMD_RDECRQ    : Opcode = Opcode(0x3c);
/* End Range operations */

/* VBucket commands */
pub const PROTOCOL_BINARY_CMD_SET_VBUCKET : Opcode = Opcode(0x3d);
pub const PROTOCOL_BINARY_CMD_GET_VBUCKET : Opcode = Opcode(0x3e);
pub const PROTOCOL_BINARY_CMD_DEL_VBUCKET : Opcode = Opcode(0x3f);
/* End VBucket commands */

/* TAP commands */
pub const PROTOCOL_BINARY_CMD_TAP_CONNECT : Opcode = Opcode(0x40);
pub const PROTOCOL_BINARY_CMD_TAP_MUTATION : Opcode = Opcode(0x41);
pub const PROTOCOL_BINARY_CMD_TAP_DELETE : Opcode = Opcode(0x42);
pub const PROTOCOL_BINARY_CMD_TAP_FLUSH : Opcode = Opcode(0x43);
pub const PROTOCOL_BINARY_CMD_TAP_OPAQUE : Opcode = Opcode(0x44);
pub const PROTOCOL_BINARY_CMD_TAP_VBUCKET_SET : Opcode = Opcode(0x45);
pub const PROTOCOL_BINARY_CMD_TAP_CHECKPOINT_START : Opcode = Opcode(0x46);
pub const PROTOCOL_BINARY_CMD_TAP_CHECKPOINT_END : Opcode = Opcode(0x47);
/* End TAP */

pub const PROTOCOL_BINARY_CMD_LAST_RESERVED : Opcode = Opcode(0xef);

/* Scrub the data */
pub const PROTOCOL_BINARY_CMD_SCRUB : Opcode = Opcode(0xf0);

/**
* Definition of the data types in the packet
* See section 3.4 Data Types
*/
#[derive(Debug)]
pub struct DataType(pub u8);
pub const PROTOCOL_BINARY_RAW_BYTES : DataType = DataType(0x00);

#[derive(Debug)]
pub struct Header {
    pub magic : Magic,
    pub opcode : Opcode,
    pub keylen : u16,
    pub extlen : u8,
    pub datatype : DataType,
    pub status : ResponseStatus,
    pub bodylen : u32,
    pub opaque : u32,
    pub cas : u64,
}

#[derive(Debug)]
pub struct Packet {
    pub header : Header,
    pub extras : Vec<u8>,
    pub key : String,
    pub value : Vec<u8>,
}

impl Header {
    pub fn new_request(opcode : Opcode, opaque : u32, extlen : usize, keylen : usize, valuelen : usize) -> Self {
        Header {
            magic : PROTOCOL_BINARY_REQ,
            opcode : opcode,
            keylen : keylen as u16,
            extlen : extlen as u8,
            datatype : PROTOCOL_BINARY_RAW_BYTES,
            status : PROTOCOL_BINARY_RESPONSE_SUCCESS,
            bodylen : (extlen + keylen + valuelen) as u32,
            opaque : opaque,
            cas : 0,
        }
    }
}

pub const HEADER_SIZE : usize = 24;

impl Packet {
    pub fn new_request_get(opaque : u32, key : String) -> Self {
        Packet {
            header : Header::new_request(PROTOCOL_BINARY_CMD_GET, opaque, 0, key.len(), 0),
            extras : Vec::new(),
            key : key,
            value : Vec::new(),
        }
    }
    pub fn new_request_set(opaque : u32, key : String, value : Vec<u8>) -> Self {
        Packet {
            header : Header::new_request(PROTOCOL_BINARY_CMD_SET, opaque, 8, key.len(), value.len()),
            extras : vec![0; 8],
            key : key,
            value : value,
        }
    }
}

