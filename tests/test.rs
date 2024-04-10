use insta::{allow_duplicates, assert_snapshot};
use rstest::rstest;
use rusqlite::types::FromSql;
use rusqlite::{Connection, Result};

#[ctor::ctor]
fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

pub struct Conn(Connection);

impl Default for Conn {
    fn default() -> Self {
        let db = Connection::open_in_memory().unwrap();
        sqlite_compressions::register_compression_functions(&db).unwrap();
        Self(db)
    }
}

impl Conn {
    pub fn sql<T: FromSql>(&self, query: &str) -> Result<T> {
        self.0.query_row_and_then(query, [], |r| r.get(0))
    }

    pub fn s(&self, func: &str, param: &str) -> String {
        let query = format!("SELECT {}", param.replace('%', func));
        match self.sql::<Option<Vec<u8>>>(&query) {
            Ok(v) => match v {
                Some(v) => hex::encode(v),
                None => "NULL".into(),
            },
            Err(e) => e.to_string(),
        }
    }

    pub fn bool(&self, func: &str, param: &str) -> String {
        let query = format!("SELECT {}", param.replace('%', func));
        match self.sql::<Option<bool>>(&query) {
            Ok(v) => match v {
                Some(v) => v.to_string(),
                None => "NULL".into(),
            },
            Err(e) => e.to_string(),
        }
    }
}

#[rstest]
#[cfg_attr(feature = "gzip", case("gzip"))]
#[cfg_attr(feature = "brotli", case("brotli"))]
#[cfg_attr(feature = "bzip2", case("bzip2"))]
#[trace]
#[test]
fn common(#[case] func: &str) {
    let c = Conn::default();
    allow_duplicates!(
        assert_snapshot!(c.s(func, "%(NULL)"), @"NULL");
        assert_snapshot!(c.s(func, "%_decode(NULL)"), @"NULL");
        assert_snapshot!(c.s(func, "%_test(NULL)"), @"NULL");

        assert_snapshot!(c.s(func, "%()"), @"Wrong number of parameters passed to query. Got 0, needed 1");
        assert_snapshot!(c.s(func, "%(1)"), @"Invalid function parameter type Integer at index 0");
        assert_snapshot!(c.s(func, "%(0.42)"), @"Invalid function parameter type Real at index 0");

        assert_snapshot!(c.s(func, "%_decode()"), @"Wrong number of parameters passed to query. Got 0, needed 1");
        assert_snapshot!(c.s(func, "%_decode(NULL)"), @"NULL");
        assert_snapshot!(c.s(func, "%_decode(1)"), @"Invalid function parameter type Integer at index 0");
        assert_snapshot!(c.s(func, "%_decode(0.42)"), @"Invalid function parameter type Real at index 0");

        assert_snapshot!(c.s(func, "%_decode(%(''))"), @"");
        assert_snapshot!(c.s(func, "%_decode(%(x''))"), @"");
        assert_snapshot!(c.s(func, "%_decode(%('a'))"), @"61");
        assert_snapshot!(c.s(func, "%_decode(%(x'00'))"), @"00");
        assert_snapshot!(c.s(func, "%_decode(%('123456789'))"), @"313233343536373839");
        assert_snapshot!(c.s(func, "%_decode(%(x'0123456789abcdef'))"), @"0123456789abcdef");

        assert_snapshot!(c.bool(func, "%_test(%(x'0123456789abcdef'))"), @"true");
        assert_snapshot!(c.bool(func, "%_test(x'0123456789abcdef')"), @"false");
    );
}

#[test]
#[cfg(feature = "gzip")]
fn gzip() {
    let c = Conn::default();
    assert_snapshot!(c.s("gzip", "%('')"), @"1f8b08000000000000ff03000000000000000000");
    assert_snapshot!(c.s("gzip", "%(x'')"), @"1f8b08000000000000ff03000000000000000000");
    assert_snapshot!(c.s("gzip", "%('a')"), @"1f8b08000000000000ff4b040043beb7e801000000");
    assert_snapshot!(c.s("gzip", "%(x'00')"), @"1f8b08000000000000ff6300008def02d201000000");
    assert_snapshot!(c.s("gzip", "%('123456789')"), @"1f8b08000000000000ff33343236313533b7b004002639f4cb09000000");
    assert_snapshot!(c.s("gzip", "%(x'0123456789abcdef')"), @"1f8b08000000000000ff6354764def5c7df63d00aed1c72808000000");

    assert_snapshot!(c.s("gzip", "%(x'0123', 0)"), @"1f8b08000000000004ff010200fdff0123cc52a5fa02000000");
    assert_snapshot!(c.s("gzip", "%(x'0123', 5)"), @"1f8b08000000000000ff63540600cc52a5fa02000000");
    assert_snapshot!(c.s("gzip", "%(x'0123', 9)"), @"1f8b08000000000002ff63540600cc52a5fa02000000");
}

#[test]
#[cfg(feature = "brotli")]
fn brotli() {
    let c = Conn::default();
    assert_snapshot!(c.s("brotli", "%('')"), @"3b");
    assert_snapshot!(c.s("brotli", "%(x'')"), @"3b");
    assert_snapshot!(c.s("brotli", "%('a')"), @"0b00806103");
    assert_snapshot!(c.s("brotli", "%(x'00')"), @"0b00800003");
    assert_snapshot!(c.s("brotli", "%('123456789')"), @"0b048031323334353637383903");
    assert_snapshot!(c.s("brotli", "%(x'0123456789abcdef')"), @"8b03800123456789abcdef03");

    assert_snapshot!(c.s("brotli", "%(x'0123', 0)"), @"8b0080012303");
    assert_snapshot!(c.s("brotli", "%(x'0123', 10)"), @"8b0080012303");
    assert_snapshot!(c.s("brotli", "%(x'0123', 99)"), @"8b0080012303");
}

#[test]
#[cfg(feature = "bzip2")]
fn bzip2() {
    let c = Conn::default();
    assert_snapshot!(c.s("bzip2", "%('')"), @"425a683617724538509000000000");
    assert_snapshot!(c.s("bzip2", "%(x'')"), @"425a683617724538509000000000");
    assert_snapshot!(c.s("bzip2", "%('a')"), @"425a683631415926535919939b6b00000001002000200021184682ee48a70a120332736d60");
    assert_snapshot!(c.s("bzip2", "%(x'00')"), @"425a6836314159265359b1f7404b00000040004000200021184682ee48a70a12163ee80960");
    assert_snapshot!(c.s("bzip2", "%('123456789')"), @"425a6836314159265359fc89191800000008003fe02000220d0c0832621e0def29c177245385090fc8919180");
    assert_snapshot!(c.s("bzip2", "%(x'0123456789abcdef')"), @"425a6836314159265359f61121f9000000555520000800020000800020000800020000a000310c08191a69933573f945dc914e14243d84487e40");

    assert_snapshot!(c.s("bzip2", "%(x'0123', 0)"), @"unwinding panic");
    assert_snapshot!(c.s("bzip2", "%(x'0123', 10)"), @"The optional second argument to bzip2() must be between 0 and 9");
    assert_snapshot!(c.s("bzip2", "%(x'0123', 99)"), @"The optional second argument to bzip2() must be between 0 and 9");
}
