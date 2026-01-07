use std::{env, process::ExitCode};

#[inline]
fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        panic!("too many arguments")
    };

    let endpoint = args.last().unwrap();

    let resp = minreq::get(endpoint).send();

    if let Err(error) = resp {
        println!("{}", error);
        return ExitCode::from(1);
    };

    let resp = resp.unwrap();
    let code = resp.status_code;

    if code > 299 {
        println!("received status code {code}");
        return ExitCode::from(1);
    };

    assert_eq!(resp.as_str().unwrap(), "ok");

    ExitCode::from(0)
}

#[cfg(test)]
mod tests {
    #[test]
    fn can_reach_google() {
        let resp = minreq::get("http://google.com").send();
        assert!(resp.is_ok())
    }

    #[test]
    fn cant_reach_random() {
        let resp = minreq::get("http://grioegnpreewfoaipn.local/fdisof").send();
        assert!(resp.is_err())
    }
}
