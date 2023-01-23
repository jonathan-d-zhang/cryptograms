use rand::prelude::*;

const KEY_LENGTH: usize = 4;

fn generate_key<R>(rng: &mut R) -> Vec<u8>
where
    R: Rng + ?Sized,
{
    let t: Vec<_> = (b'a'..=b'z').collect();
    t.choose_multiple(rng, KEY_LENGTH).copied().collect()
}

fn matmul(plaintext: &[u8], key: Vec<Vec<u8>>) -> Vec<u8> {
    log::trace!("Matmulling plaintext{:?} with key={:?}", String::from_utf8_lossy(plaintext), key.clone().into_iter().map(|r| String::from_utf8(r).unwrap()).collect::<Vec<_>>());
    // our plaintexts are limited to 160 bytes, naive algo is fine
    let mut result = Vec::new();

    for i in (0..plaintext.len()).step_by(key.len()) {
        for j in 0..key.len() {
            let mut s: u32 = 0;
            for k in 0..key.len() {
                log::trace!("    i={i}, j={j}, k={k}, s={s}");
                s += (plaintext[i + k] - b'a') as u32 * (key[j][k] - b'a') as u32;
            }
            result.push((s % 26) as u8 + b'a');
            log::trace!("    Added {}", ((s % 26) as u8 + b'a') as char);
        }
    }

    log::trace!("    Final Result: {:?}", String::from_utf8(result.clone()));

    result
}

fn is_perfect_square(n: usize) -> bool {
    let mut i = 1;
    loop {
        let t = i * i;
        if t == n {
            return true;
        } else if t > n {
            return false
        }

        i += 1;
    }
}

///
pub fn hill<R>(plaintext: &str, key: Option<String>, rng: &mut R) -> String
where
    R: Rng + ?Sized,
{
    let key = match key {
        Some(k) => k.bytes().collect(),
        None => generate_key(rng),
    };

    log::debug!("Hill: key={:?}", String::from_utf8(key.clone()).unwrap());

    let n = key.len();

    // key must be a perfect square
    if !is_perfect_square(n) {
        // TODO: change return type of cipher functions to Result
        log::warn!("Key length {} is not a perfect square", n);
        panic!("Key length must be a perfect square");
    }

    let side_length = (n as f64).sqrt() as usize;

    // remove non-letters
    let mut filtered = String::with_capacity(plaintext.len());
    for c in plaintext.chars() {
        if c.is_ascii_alphabetic() {
            filtered.push(c.to_ascii_lowercase());
        }
    }

    // if the length is not divisible by `side_length`, pad with 'z'
    let to_pad = filtered.len() % side_length;

    filtered.extend("z".repeat(to_pad).chars());

    // convert 1-d key into square matrix
    let mut bytes = key.into_iter();
    let mut matrix = vec![vec![0; side_length]; side_length];
    for i in 0..side_length {
        for j in 0..side_length {
            matrix[i][j] = bytes.next().unwrap();
        }
    }

    let r = matmul(filtered.as_bytes(), matrix);

    String::from_utf8(r).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::mock::StepRng;

    #[test]
    fn test_generate_key() {
        let mut rng = StepRng::new(0, 1);
        let res = generate_key(&mut rng);

        assert_eq!(res, vec![b'x', b'y', b'z', b'a']);
    }

    #[test]
    fn test_hill() {
        let mut rng = StepRng::new(0, 1);
        let res = hill("abcd", Some("abcd".into()), &mut rng);

        assert_eq!(res, "bddn");
    }

    #[test]
    fn test_matmul() {
        let plaintext = b"abcd";
        let key = vec![vec![b'a', b'b'], vec![b'c', b'd']];

        let res = matmul(plaintext, key);
        println!("{:?}", String::from_utf8(res.clone()).unwrap());

        assert_eq!(res, vec![b'b', b'd', b'd', b'n'])
    }

    #[test]
    fn test_is_perfect_square() {
        assert!(is_perfect_square(4));
        assert!(is_perfect_square(9));
        assert!(!is_perfect_square(15));
    }

}
