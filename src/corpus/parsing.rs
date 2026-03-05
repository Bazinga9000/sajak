use super::trie::CorpusNode;
use nom::IResult;
use nom::Parser;

const CHAR_IDS: [char; 38] = [
    '\0', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
    'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', ' ',
];

struct LabelByte {
    pub label: char,
    pub terminal: bool,
    pub has_children: bool,
}

fn read_label_byte(byte: u8) -> LabelByte {
    LabelByte {
        label: CHAR_IDS[(byte >> 2) as usize],
        terminal: byte & 0b_000000_1_0 > 0,
        has_children: byte & 0b000000_0_1 > 0,
    }
}

pub static mut total_calls: usize = 0;

fn parse_label_byte(input: &[u8]) -> IResult<&[u8], LabelByte> {
    (nom::number::complete::u8)
        .map(read_label_byte)
        .parse(input)
}

fn parse_efficient_u64(input: &[u8]) -> IResult<&[u8], u64> {
    let (mut rest, discriminator) = nom::number::complete::u8.parse(input)?;
    if discriminator < 249 {
        return Ok((rest, discriminator as u64));
    }

    let mut integer = [0; 8];
    for i in 0..(discriminator - 247) as usize {
        (rest, integer[i]) = nom::number::complete::u8.parse(rest)?;
    }

    Ok((rest, u64::from_le_bytes(integer)))
}

pub fn parse_node_at(offset: usize, input: &[u8]) -> IResult<&[u8], CorpusNode> {
    unsafe {
        total_calls += 1;
    }
    let (rest, labelbyte) = parse_label_byte(&input[offset..])?;
    let (rest, frequency) = parse_efficient_u64(rest)?;
    if !labelbyte.has_children {
        Ok((
            rest,
            CorpusNode {
                label: labelbyte.label,
                frequency,
                own_frequency: frequency,
                is_terminal: labelbyte.terminal,
                child_offsets: vec![],
            },
        ))
    } else {
        let (rest, num_children) = nom::number::complete::u8.parse(rest)?;
        let (rest, offsets) = nom::multi::count(
            nom::combinator::map(parse_efficient_u64, |n| offset - (n as usize)),
            num_children as usize,
        )
        .parse(rest)?;
        let mut sum_child_frequencies: u64 = 0;
        for child_offset in offsets.iter() {
            sum_child_frequencies += parse_efficient_u64(&input[*child_offset+1..])?.1;
        }
        Ok((
            rest,
            CorpusNode {
                label: labelbyte.label,
                frequency,
                own_frequency: frequency - sum_child_frequencies,
                is_terminal: labelbyte.terminal,
                child_offsets: offsets,
            },
        ))
    }
}
