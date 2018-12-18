use ff::{Field, PrimeField, PrimeFieldRepr, BitIterator};
use sapling_crypto::jubjub::{JubjubEngine};
use bellman::{ConstraintSystem, SynthesisError};

use super::Assignment;
use super::boolean;
use super::num::{AllocatedNum, Num};
use super::float_point::{parse_with_exponent_le, convert_to_float};
use super::baby_eddsa::EddsaSignature;
use super::plasma_constants;


// count a number of non-zero bits in a bit decomposition
pub fn count_number_of_ones<E, CS>(
        mut cs: CS,
        a: &[boolean::Boolean]
    ) -> Result<AllocatedNum<E>, SynthesisError>
        where E: JubjubEngine,
        CS: ConstraintSystem<E>
{
    let mut counter = Num::zero();
    for bit in a.iter() {
        counter = counter.add_bool_with_coeff(CS::one(), &bit, E::Fr::one());
    }

    let result = AllocatedNum::alloc(
        cs.namespace(|| "number of zeroes number"), 
        || Ok(*counter.get_value().get()?)
    )?;

    cs.enforce(
        || "pack number of ones",
        |lc| lc + result.get_variable(),
        |lc| lc + CS::one(),
        |_| counter.lc(E::Fr::one())
    );

    Ok(result)
}

pub fn allocate_audit_path<E, CS> (
    mut cs: CS,
    audit_path: Vec<Option<E::Fr>>,
) -> Result<Vec<AllocatedNum<E>>, SynthesisError> 
    where E: JubjubEngine,
          CS: ConstraintSystem<E>
{
    let mut allocated = vec![];
    for (i, e) in audit_path.into_iter().enumerate() {
        let path_element = AllocatedNum::alloc(
            cs.namespace(|| format!("path element{}", i)),
            || {
                Ok(*e.get()?)
            }
        )?;
        allocated.push(path_element);
    }

    Ok(allocated)
}

pub fn append_packed_public_key(
    content: & mut Vec<boolean::Boolean>,
    x_bits: Vec<boolean::Boolean>,
    y_bits: Vec<boolean::Boolean>,
) 
{
    assert_eq!(*plasma_constants::FR_BIT_WIDTH - 1, y_bits.len());
    assert_eq!(1, x_bits.len());
    content.extend(y_bits);
    content.extend(x_bits);
}

pub fn le_bit_vector_into_field_element<P: PrimeField>
    (bits: &Vec<bool>) -> P
{
    // double and add
    let mut fe = P::zero();
    let mut base = P::one();

    for bit in bits {
        if *bit {
            fe.add_assign(&base);
        }
        base.double();
    }

    fe
    // // TODO remove representation length hardcode
    // let mut bytes = [0u8; 32];

    // let byte_chunks = bits.chunks(8);

    // for (i, byte_chunk) in byte_chunks.enumerate()
    // {
    //     let mut byte = 0u8;
    //     for (j, bit) in byte_chunk.into_iter().enumerate()
    //     {
    //         if *bit {
    //             byte |= 1 << j;
    //         }
    //     }
    //     bytes[i] = byte;
    // }

    // let mut repr : P::Repr = P::zero().into_repr();
    // repr.read_le(&bytes[..]).expect("interpret as field element");

    // let field_element = P::from_repr(repr).unwrap();

    // field_element
}

pub fn be_bit_vector_into_bytes
    (bits: &Vec<bool>) -> Vec<u8>
{
    let mut bytes: Vec<u8> = vec![];

    let byte_chunks = bits.chunks(8);

    for byte_chunk in byte_chunks
    {
        let mut byte = 0u8;
        // pack just in order
        for (i, bit) in byte_chunk.into_iter().enumerate()
        {
            if *bit {
                byte |= 1 << (7 - i);
            }
        }
        bytes.push(byte);
    }

    bytes
}

pub fn le_bit_vector_into_bytes
    (bits: &Vec<bool>) -> Vec<u8>
{
    let mut bytes: Vec<u8> = vec![];

    let byte_chunks = bits.chunks(8);

    for byte_chunk in byte_chunks
    {
        let mut byte = 0u8;
        // pack just in order
        for (i, bit) in byte_chunk.into_iter().enumerate()
        {
            if *bit {
                byte |= 1 << i;
            }
        }
        bytes.push(byte);
    }

    bytes
}

pub fn encode_fs_into_fr<E: JubjubEngine>(input: E::Fs)
    -> E::Fr 
{
    let mut fs_le_bits: Vec<bool> = BitIterator::new(input.into_repr()).collect();
    fs_le_bits.reverse();

    let converted = le_bit_vector_into_field_element::<E::Fr>(&fs_le_bits);

    converted
}