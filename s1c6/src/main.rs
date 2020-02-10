use std::io::{BufReader,prelude::*};
use std::fs::File;

extern crate hamming;
extern crate base64;

struct ProbableComputations {
    key_byte: u8,
    ascii_score: u64,
    decrypted_content: Vec<u8>
}

impl ProbableComputations {
    fn new() -> ProbableComputations {
        ProbableComputations {
            key_byte: 0x00,
            ascii_score: 0,
            decrypted_content: Vec::<u8>::new(),
        }
    }
}

struct DistanceAvg {
    keysize: usize,
    avg_dist: u64,
}

fn check_ascii_bytes(xstr: &Vec<u8>, kb: u8) -> Option<ProbableComputations> {
    let mut chk_bytes_obj = ProbableComputations::new();
    chk_bytes_obj.key_byte = kb;

    for i in 0..xstr.len() {
        if (xstr[i] > 0x1f) && (xstr[i] < 0x7e) {
            chk_bytes_obj.ascii_score += 1;
        }
        if (xstr[i] >= 0x00) && (xstr[i] < 0x20) {
            if chk_bytes_obj.ascii_score > 1 {
                chk_bytes_obj.ascii_score -= 1
            }
        }
        if (xstr[i] > 0x21) && (xstr[i] < 0x30) {
            if chk_bytes_obj.ascii_score > 1 {
                chk_bytes_obj.ascii_score -= 1;
            }
        }
        if (xstr[i] > 0x39) && (xstr[i] < 0x41) {
            if chk_bytes_obj.ascii_score > 1 {
                chk_bytes_obj.ascii_score -= 1;
            }
        }
        if xstr[i] == 0x20 {
            chk_bytes_obj.ascii_score += 3;
        }
        if (xstr[i] > 0x2f) && (xstr[i] < 0x3a) {
            chk_bytes_obj.ascii_score += 2;
        }
        if (xstr[i] > 0x40) && (xstr[i] < 0x5b) {
            chk_bytes_obj.ascii_score += 4;
        }
        if (xstr[i] > 0x60) && (xstr[i] < 0x7b) {
            chk_bytes_obj.ascii_score += 5;
        }
    }

    Some(ProbableComputations {
        key_byte: chk_bytes_obj.key_byte,
        ascii_score: chk_bytes_obj.ascii_score,
        decrypted_content: xstr.to_vec(),
    })
}

fn getNormKeySize(ba1: &Vec<u8>, ba2: &Vec<u8>) -> f64 {
    //println!("pew1 {:x} pew2 {:x}",ba1,ba2);
    let keysize = ba1.len();
    let hd: u64 = hamming::distance(ba1,ba2);
    let NormKeySize = hd as f64 / keysize as f64;

    NormKeySize
}
/*
 * Get list of hamdist / keysize
 * Get average of each Normalization
 * Pop off of vector until size of block is larger than amount of vector elements left
 * Put average and keysize into a DistanceAvg structure
 * The DistanceAvg structure with the lowest avg distance is the correct keysize
 */
fn getKeySize(ct: &Vec<u8>) -> u64 {
    println!("Getting keysize");
    let mut NormKeyScore: u64 = 100;
    let mut KeyScore: u64 = 0;
    let mut distance_array: Vec<f64> = Vec::new();
    let mut ct_new = ct.clone();
    let mut distance_avgs: Vec<DistanceAvg> = Vec::new();

    //println!("Starting iter");
    for i in 2..41 {
        
        let mut ct_iter = ct_new.clone();
        //println!("Cloned ct_iter");

        //println!("Starting chunk iter");
        let mut ct_iter_obj = ct_iter.chunks(i);

        loop {
            let ch1 = match ct_iter_obj.next() {
                Some(s) => s,
                None => {break;},
            };
            let ch1_vec = ch1.to_vec();
            //println!("Got chunk: {:?}",ch1_vec);
            if ch1_vec.len() != i {
                //println!("Chunk length: {}",ch1_vec.len());
                break;
            }
            let ch2 = match ct_iter_obj.next() {
                Some(s) => s,
                None => {break;},
            };
            let ch2_vec = ch2.to_vec();
            //println!("Got chunk2: {:?}",ch2_vec);
            if ch2_vec.len() != i {
                //println!("Chunk length: {}",ch2_vec.len());
                break;
            }

            distance_array.push(getNormKeySize(&ch1_vec, &ch2_vec));
            //println!("normkeysize complete");
        }

        //println!("Setting object params");
        let da_len = distance_array.len();
        let sum: f64 = distance_array.iter().sum();
        distance_array.clear();
        /*
        for d in &distance_array {
            sum += d;
        }*/

        let avg = sum as f64 / da_len as f64;
        println!("Got avg: {} - {}",avg,i);
        distance_avgs.push(DistanceAvg{
            keysize: i,
            avg_dist: avg as u64,
        });
    }
    //println!("Sorting..");
    distance_avgs.sort_by(|a,b| a.avg_dist.cmp(&b.avg_dist));
    println!("Keysize: {} - Average: {}",distance_avgs[0].keysize,distance_avgs[0].avg_dist);
    distance_avgs[0].keysize as u64
}

fn read_lines() -> Vec<u8> {

    let mut f = File::open("6.txt").unwrap();
    let mut buffer: Vec<u8> = Vec::new();
    f.read_to_end(&mut buffer).unwrap();

    buffer
}

fn main() {

    let mut ct: Vec<u8> = Vec::new();

    ct = base64::decode(&read_lines()).unwrap();

    println!("CipherText Length: {}", ct.len());

    let keysize = getKeySize(&ct); //<- Cant get working

    //println!("KeySize: {}",keysize);

    let mut BlockArray: Vec<Vec<u8>> = Vec::new();

    for i in 0..keysize {
        BlockArray.push(Vec::new());
    }



    let ct_chunks = ct.chunks(keysize as usize);

    let chunk_amount = ct_chunks.len();
    
    let mut end_count = ct.len();


    for ct_chunk in ct_chunks {
        for i in 0..keysize {
            if end_count == 0 {
                break;
            } else {
                BlockArray[i as usize].push(ct_chunk[i as usize]);
                end_count -= 1;
            }
        }
    }

    let mut encryption_key: Vec<u8> = Vec::<u8>::new();
    let mut prob_comp = ProbableComputations::new();

    println!("Decrypting {} blocks with keysize {}\n",chunk_amount,keysize);

    for line in BlockArray {

        let mut xordbytes: Vec<u8> = Vec::<u8>::new();

        for i in 0..256 {
            for j in 0..line.len() {
                xordbytes.push(line[j] ^ i as u8);
            }
            match check_ascii_bytes(&xordbytes,i as u8) {
                Some(pc) => {if pc.ascii_score > prob_comp.ascii_score{prob_comp = pc;}},
                None => {}
            }
            xordbytes.clear();
        }
        println!("Key Byte Score: {}\tKey Byte: {:02X}",prob_comp.ascii_score,prob_comp.key_byte);
        encryption_key.push(prob_comp.key_byte);
        prob_comp = ProbableComputations::new();
    }
    
    let mut kb_count = encryption_key.len();
    print!("\nKey Found: [");
    for kb in &encryption_key {
        if kb_count != 1 {
            print!("{:02X}:",kb);
            kb_count -= 1;
        } else {
            print!("{:02X}",kb);
        }
    }
    print!("]\n");
    println!("Plaintext Key: [{}]",String::from_utf8(encryption_key).unwrap());
}