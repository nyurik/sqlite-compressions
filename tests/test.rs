#![expect(clippy::unwrap_used)]

use insta::assert_snapshot;
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
        self.q(&param.replace('%', func))
    }

    pub fn q(&self, query: &str) -> String {
        let query = format!("SELECT {query}");
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

#[rstest::rstest]
#[cfg_attr(feature = "gzip", case("gzip"))]
#[cfg_attr(feature = "brotli", case("brotli"))]
#[cfg_attr(feature = "bzip2", case("bzip2"))]
#[trace]
#[test]
#[cfg(any(feature = "brotli", feature = "bzip2", feature = "gzip"))]
fn common(#[case] func: &str) {
    let c = Conn::default();
    insta::allow_duplicates!(
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
    assert_snapshot!(c.q("gzip('')"), @"1f8b08000000000000ff03000000000000000000");
    assert_snapshot!(c.q("gzip(x'')"), @"1f8b08000000000000ff03000000000000000000");
    assert_snapshot!(c.q("gzip('a')"), @"1f8b08000000000000ff4b040043beb7e801000000");
    assert_snapshot!(c.q("gzip(x'00')"), @"1f8b08000000000000ff6300008def02d201000000");
    assert_snapshot!(c.q("gzip('123456789')"), @"1f8b08000000000000ff33343236313533b7b004002639f4cb09000000");
    assert_snapshot!(c.q("gzip(x'0123456789abcdef')"), @"1f8b08000000000000ff6354764def5c7df63d00aed1c72808000000");

    assert_snapshot!(c.q("gzip(x'0123', 0)"), @"1f8b08000000000004ff010200fdff0123cc52a5fa02000000");
    assert_snapshot!(c.q("gzip(x'0123', 5)"), @"1f8b08000000000000ff63540600cc52a5fa02000000");
    assert_snapshot!(c.q("gzip(x'0123', 9)"), @"1f8b08000000000002ff63540600cc52a5fa02000000");
}

#[test]
#[cfg(feature = "brotli")]
fn brotli() {
    let c = Conn::default();
    assert_snapshot!(c.q("brotli('')"), @"3b");
    assert_snapshot!(c.q("brotli(x'')"), @"3b");
    assert_snapshot!(c.q("brotli('a')"), @"0b00806103");
    assert_snapshot!(c.q("brotli(x'00')"), @"0b00800003");
    assert_snapshot!(c.q("brotli('123456789')"), @"0b048031323334353637383903");
    assert_snapshot!(c.q("brotli(x'0123456789abcdef')"), @"8b03800123456789abcdef03");

    assert_snapshot!(c.q("brotli(x'0123', 0)"), @"8b0080012303");
    assert_snapshot!(c.q("brotli(x'0123', 10)"), @"8b0080012303");
    assert_snapshot!(c.q("brotli(x'0123', 99)"), @"8b0080012303");
}

#[test]
#[cfg(feature = "bzip2")]
fn bzip2() {
    let c = Conn::default();
    assert_snapshot!(c.q("bzip2('')"), @"425a683617724538509000000000");
    assert_snapshot!(c.q("bzip2(x'')"), @"425a683617724538509000000000");
    assert_snapshot!(c.q("bzip2('a')"), @"425a683631415926535919939b6b00000001002000200021184682ee48a70a120332736d60");
    assert_snapshot!(c.q("bzip2(x'00')"), @"425a6836314159265359b1f7404b00000040004000200021184682ee48a70a12163ee80960");
    assert_snapshot!(c.q("bzip2('123456789')"), @"425a6836314159265359fc89191800000008003fe02000220d0c0832621e0def29c177245385090fc8919180");
    assert_snapshot!(c.q("bzip2(x'0123456789abcdef')"), @"425a6836314159265359f61121f9000000555520000800020000800020000800020000a000310c08191a69933573f945dc914e14243d84487e40");

    // errors
    assert_snapshot!(c.q("bzip2(x'0123', 0)"), @"The optional second argument to bzip2() must be between 1 and 9");
    assert_snapshot!(c.q("bzip2(x'0123', 10)"), @"The optional second argument to bzip2() must be between 1 and 9");
    assert_snapshot!(c.q("bzip2(x'0123', 99)"), @"The optional second argument to bzip2() must be between 1 and 9");
}

#[test]
#[cfg(feature = "bsdiff4")]
fn bsdiff4() {
    let c = Conn::default();
    assert_snapshot!(c.q("bsdiff4('', '')"), @"42534449464634300e000000000000000e000000000000000000000000000000425a683617724538509000000000425a683617724538509000000000425a683617724538509000000000");
    assert_snapshot!(c.q("bsdiff4(x'', x'')"), @"42534449464634300e000000000000000e000000000000000000000000000000425a683617724538509000000000425a683617724538509000000000425a683617724538509000000000");
    assert_snapshot!(c.q("bsdiff4('a', '')"), @"42534449464634300e000000000000000e000000000000000000000000000000425a683617724538509000000000425a683617724538509000000000425a683617724538509000000000");
    assert_snapshot!(c.q("bsdiff4(x'00', '')"), @"42534449464634300e000000000000000e000000000000000000000000000000425a683617724538509000000000425a683617724538509000000000425a683617724538509000000000");
    assert_snapshot!(c.q("bsdiff4('123456789', '123456789')"), @"42534449464634302b0000000000000025000000000000000900000000000000425a6836314159265359439c5a03000000e00040200c00200030cd341268369327177245385090439c5a03425a6836314159265359752890670000004000420020002100828317724538509075289067425a683617724538509000000000");
    assert_snapshot!(c.q("bsdiff4(x'0123456789abcdef', x'0123456789abcdef')"), @"42534449464634302b0000000000000025000000000000000800000000000000425a68363141592653591533c7b1000000e00040400c00200030cd3412683693271772453850901533c7b1425a683631415926535996fb44a60000004000440020002100828317724538509096fb44a6425a683617724538509000000000");
    assert_snapshot!(c.q("bsdiff4('1234', '5678349A')"), @"42534449464634302a000000000000000e000000000000000800000000000000425a68363141592653591769093d00000140004c402000219a68334d32b6c078bb9229c28480bb4849e8425a683617724538509000000000425a683631415926535986c673c30000010c000fe0200020002190c210c08795392f8bb9229c2848436339e180");
    assert_snapshot!(c.q("bsdiff4(x'1234', x'5678349A')"), @"42534449464634302c000000000000000e000000000000000400000000000000425a68363141592653590dcec6b900000140005c00200030cd34129ea7a357029ee1772453850900dcec6b90425a683617724538509000000000425a6836314159265359ff2387120000008aa004000100004000102000219a68334d32bc5dc914e14243fc8e1c48");

    assert_snapshot!(c.q("bspatch4('', bsdiff4('', ''))"), @"");
    assert_snapshot!(c.q("bspatch4(x'', bsdiff4(x'', x''))"), @"");
    assert_snapshot!(c.q("bspatch4('a', bsdiff4('a', ''))"), @"");
    assert_snapshot!(c.q("bspatch4(x'00', bsdiff4(x'00', ''))"), @"");
    assert_snapshot!(c.q("bspatch4('123456789', bsdiff4('123456789', '123456789'))"), @"313233343536373839");
    assert_snapshot!(c.q("bspatch4(x'0123456789abcdef', bsdiff4(x'0123456789abcdef', x'0123456789abcdef'))"), @"0123456789abcdef");
    assert_snapshot!(c.q("bspatch4('1234', bsdiff4('1234', '5678349A'))"), @"3536373833343941");
    assert_snapshot!(c.q("bspatch4(x'1234', bsdiff4(x'1234', x'5678349A'))"), @"5678349a");

    // nulls
    assert_snapshot!(c.q("bsdiff4(NULL, NULL)"), @"NULL");
    assert_snapshot!(c.q("bsdiff4('abc', NULL)"), @"NULL");
    assert_snapshot!(c.q("bsdiff4(NULL, 'abc')"), @"NULL");
    assert_snapshot!(c.q("bspatch4(NULL, NULL)"), @"NULL");
    assert_snapshot!(c.q("bspatch4('abc', NULL)"), @"NULL");
    assert_snapshot!(c.q("bspatch4(NULL, 'abc')"), @"NULL");

    // errors
    assert_snapshot!(c.q("bsdiff4(x'0123')"), @"wrong number of arguments to function bsdiff4()");
    assert_snapshot!(c.q("bsdiff4(x'0123', x'4567', x'89')"), @"wrong number of arguments to function bsdiff4()");
    assert_snapshot!(c.q("bspatch4(x'0123')"), @"wrong number of arguments to function bspatch4()");
    assert_snapshot!(c.q("bspatch4(x'0123', x'4567', x'89')"), @"wrong number of arguments to function bspatch4()");
}

#[test]
#[cfg(feature = "bsdiffraw")]
fn bsdiffraw() {
    let c = Conn::default();
    assert_snapshot!(c.q("bsdiffraw('', '')"), @"");
    assert_snapshot!(c.q("bsdiffraw(x'', x'')"), @"");
    assert_snapshot!(c.q("bsdiffraw('a', '')"), @"");
    assert_snapshot!(c.q("bsdiffraw(x'00', '')"), @"");
    assert_snapshot!(c.q("bsdiffraw('123456789', '123456789')"), @"090000000000000000000000000000000900000000000080000000000000000000");
    assert_snapshot!(c.q("bsdiffraw(x'0123456789abcdef', x'0123456789abcdef')"), @"0800000000000000000000000000000008000000000000800000000000000000");
    assert_snapshot!(c.q("bsdiffraw('1234', '5678349A')"), @"0000000000000000080000000000000003000000000000003536373833343941");
    assert_snapshot!(c.q("bsdiffraw(x'1234', x'5678349A')"), @"0000000000000000040000000000000001000000000000005678349a");

    assert_snapshot!(c.q("bspatchraw('', bsdiffraw('', ''))"), @"");
    assert_snapshot!(c.q("bspatchraw(x'', bsdiffraw(x'', x''))"), @"");
    assert_snapshot!(c.q("bspatchraw('a', bsdiffraw('a', ''))"), @"");
    assert_snapshot!(c.q("bspatchraw(x'00', bsdiffraw(x'00', ''))"), @"");
    assert_snapshot!(c.q("bspatchraw('123456789', bsdiffraw('123456789', '123456789'))"), @"313233343536373839");
    assert_snapshot!(c.q("bspatchraw(x'0123456789abcdef', bsdiffraw(x'0123456789abcdef', x'0123456789abcdef'))"), @"0123456789abcdef");
    assert_snapshot!(c.q("bspatchraw('1234', bsdiffraw('1234', '5678349A'))"), @"3536373833343941");
    assert_snapshot!(c.q("bspatchraw(x'1234', bsdiffraw(x'1234', x'5678349A'))"), @"5678349a");

    // nulls
    assert_snapshot!(c.q("bsdiffraw(NULL, NULL)"), @"NULL");
    assert_snapshot!(c.q("bsdiffraw('abc', NULL)"), @"NULL");
    assert_snapshot!(c.q("bsdiffraw(NULL, 'abc')"), @"NULL");
    assert_snapshot!(c.q("bspatchraw(NULL, NULL)"), @"NULL");
    assert_snapshot!(c.q("bspatchraw('abc', NULL)"), @"NULL");
    assert_snapshot!(c.q("bspatchraw(NULL, 'abc')"), @"NULL");

    // errors
    assert_snapshot!(c.q("bsdiffraw(x'0123')"), @"wrong number of arguments to function bsdiffraw()");
    assert_snapshot!(c.q("bsdiffraw(x'0123', x'4567', x'89')"), @"wrong number of arguments to function bsdiffraw()");
    assert_snapshot!(c.q("bspatchraw(x'0123')"), @"wrong number of arguments to function bspatchraw()");
    assert_snapshot!(c.q("bspatchraw(x'0123', x'4567', x'89')"), @"wrong number of arguments to function bspatchraw()");
}
