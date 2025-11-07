pub use crate::worker::SimpBuilder;

mod worker;

#[cfg(test)]
mod tests {
    use super::*;

    fn concat_strings((x1, x2): (impl Into<String>, impl Into<String>)) -> String {
        format!("{}{}", x1.into(), x2.into())
    }

    #[test]
    fn multiple_parameters_test() {
        // let wc = SimpBuilder::register(5, concat_strings);
        let wc = SimpBuilder::register(2, concat_strings)
            .and_then(|x| println!("{x}"))
            .spawn();

        let string_pairs = vec![
            ("Hello", "There"),
            ("Smay", "Ful"),
            ("John", "Birdwell"),
            ("Aeskul", "Styr")
        ];

        // let res = wc.send(("Hello", "There")).unwrap();

        // std::thread::sleep(std::time::Duration::from_millis(100));
        // println!("{:?}", wc.get(res));

        // let _ = string_pairs.into_iter().map(|x| wc.send(x)).collect::<Vec<_>>();
        for pair in &string_pairs {
            wc.send(pair.clone());
        }
        std::thread::sleep(std::time::Duration::from_secs(1));

        // for n in 0..string_pairs.len() {
        //     println!("{:?}", wc.get(n));
        // }
    }
}
