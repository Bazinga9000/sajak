use super::trie::CorpusNode;
use nom::IResult;
use nom::Parser;

const CHAR_IDS: [char; 38] = [
    '\0', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
    'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', ' ',
];



pub struct LabelByte {
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

fn compute_u64_size(num: u64) -> usize {
    if num < 249 {
        1
    } else {
        1 + ((64 - num.leading_zeros()) as usize + 7) / 8 // ceiling trick
    }
}

pub fn read_label_and_frequency(offset: usize, input: &[u8]) -> IResult<&[u8], (LabelByte, u64)> {
    let (rest, labelbyte) = parse_label_byte(&input[offset..])?;
    let (rest, frequency) = parse_efficient_u64(rest)?;
    Ok((rest, (labelbyte, frequency)))
}

pub fn parse_node_at(offset: usize, input: &[u8]) -> IResult<&[u8], CorpusNode> {
    let (rest, (labelbyte, frequency)) = read_label_and_frequency(offset, input)?;
    if !labelbyte.has_children {
        Ok((
            rest,
            CorpusNode {
                offset,
                label: labelbyte.label,
                frequency,
                own_frequency: frequency,
                is_terminal: labelbyte.terminal,
                num_children: 0,
                child_offset_loc: 0,
            },
        ))
    } else {
        let (rest, own_frequency) = parse_efficient_u64(rest)?;
        let (rest, num_children) = nom::number::complete::u8.parse(rest)?;
        Ok((
            rest,
            CorpusNode {
                offset,
                label: labelbyte.label,
                frequency,
                own_frequency,
                is_terminal: labelbyte.terminal,
                num_children,
                child_offset_loc: offset + 2 + compute_u64_size(frequency) + compute_u64_size(own_frequency),
            },
        ))
    }
}

pub fn children_offsets<'a>(node: &'a CorpusNode, input: &'a [u8]) -> IResult<&'a [u8], Vec<usize>> {
    nom::multi::count(
        nom::combinator::map(parse_efficient_u64, |n| node.offset - (n as usize)),
        node.num_children as usize,
    ).parse(&input[node.child_offset_loc..])
}