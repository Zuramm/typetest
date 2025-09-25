use itertools::Itertools;
use rand::{
    seq::{IndexedRandom, SliceRandom},
    Rng,
};

pub fn random<R>(rng: &mut R, words: &Vec<&str>, n: usize) -> String
where
    R: Rng + ?Sized,
{
    #[allow(unstable_name_collisions)]
    words
        .choose_multiple(rng, n)
        .cloned()
        .intersperse(" ")
        .collect()
}

pub fn permutate<R>(
    rng: &mut R,
    mut words: Vec<&str>,
    combination: usize,
    repetition: usize,
) -> Vec<String>
where
    R: Rng + ?Sized,
{
    words.shuffle(rng);

    words
        .iter()
        .chunks(combination)
        .into_iter()
        .map(|mut chunk| {
            let mut test = chunk.join(" ");
            test.push(' ');
            test = test.repeat(repetition);
            test.pop();
            test
        })
        .collect()
}
