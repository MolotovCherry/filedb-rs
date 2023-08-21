@0xe91d31c4da29fad0;

const version :UInt32 = 1;

struct Header @0xc8b3480e0916ca2a {
    # used for keeping track of file's version
    # can be used for compatibility
    version @0 :UInt32;
}

struct Blocks @0x99d69119b2318c80 {
    blocks @0 :List(Block);
}

struct Block @0xb94cb2ef82c2b514 {
    union {
        # a block describing the block's data
        index @0 :IndexBlock;
        data @1 :DataBlock;
    }

    #
    # IndexBlock
    #
    struct IndexBlock @0xcb378bf106b275e2 {
        blockMetadata @0 :List(MetadataBlock);
    }

    # whether the block is free or used
    struct MetadataBlock @0xebd4ada1c9e1b004 {
        # pointer to the block this metadatablock describes
        block @0 :AnyPointer;
        status @1 :BlockStatus;
        nextMetadataBlock :union {
            none @2 :Void;
            # points to next MetadataBlock
            some @3 :AnyPointer;
        }

        enum BlockStatus @0xaf22fe0a4aa194db {
            inUse @0;
            free @1;
        }
    }
    #
    #
    #

    #
    # DataBlock
    #
    struct DataBlock @0xcd0198d8324cd8f7 {
        # the key this data block belongs too
        # only defined for the first block
        key :union {
            none @0 :Void;
            some @1 :Data;
        }

        # the type id this block belongs to
        # only needs to be defined for the very first block
        typeId :union {
            none @2 :Void;
            some @3 :UInt64;
        }

        # the crc of the entire data chunk (encompasses all data of all blocks together)
        # only needs to be defined on first block
        crc :union {
            none @4 :Void;
            some @5 :UInt32;
        }

        # the data this block holds
        data @6 :Data;

        # if there is a next block in the chain, a pointer to it
        nextDataBlock :union {
            none @7 :Void;
            # points to next DataBlock
            some @8 :AnyPointer;
        }

        # a pointer to the first data block if this is not the first one
        firstDataBlock :union {
            none @9 :Void;
            # points to first DataBlock in chain
            some @10 :AnyPointer;
        }
    }
    #
    #
    #
}
