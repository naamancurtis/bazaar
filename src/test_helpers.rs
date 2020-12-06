use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::{
    auth::authorize::encode_jwt,
    models::{Claims, CustomerType, TokenType},
};

pub fn create_valid_jwt_token(token_type: TokenType) -> (String, Claims) {
    let iat = Utc::now();
    let exp = iat + Duration::minutes(15);
    let count = if token_type == TokenType::Access {
        None
    } else {
        Some(0)
    };
    let claims = Claims {
        sub: Some(Uuid::new_v4()),
        customer_type: CustomerType::Known,
        cart_id: Uuid::new_v4(),
        exp: exp.timestamp() as usize,
        iat: iat.timestamp() as usize,
        id: None,
        count,
        token_type,
    };
    let token = encode_jwt(&claims, token_type).unwrap();
    (token, claims)
}

// These keys are for local unit tests only, and aren't the ones used in the app
pub fn set_token_env_vars_for_tests() {
    use std::env::set_var;
    set_var(
        "ACCESS_TOKEN_PRIVATE_KEY",
        "-----BEGIN PRIVATE KEY-----
MIIEvAIBADANBgkqhkiG9w0BAQEFAASCBKYwggSiAgEAAoIBAQDAmbrasmuzIc/Y
w2EoEMZTnbQfCgCLd1LKB4twW8OALLsy5y4pziCMrnXrJfxisY/uL6B5OiUOIsPU
H79YARyiDyPIdoiVkgfJtGi5Jy2PVq/7Ej3bhI/Gz0TTT/mpiaSgNAoDeedLYnze
dzEg0jBaEhLAZEPB7/9xa2/4YHAF0cD+uhCeJ06QPjL5Yw/3UTbq+6RJILGJbxKF
kwdneNVMZHYc/Kaih9EmemPaxToNCz9AMwzU4fp0clkj94Zi3xplnAOSjvxdCkBp
87hzHp0PN7jYqhNwYVn2J4u8PKpT+aUFBxJOIpGDvZ6UNLDWy3yahCav2B/zGQsn
z6uAC0ZXAgMBAAECggEAfGuTQy6Z1qnUHKLzA6/6fw/UyWxrt60I5YyELryJCidb
c2HW95i6fEdD0/nBFnzAj01jLI08XOpmeYVc1dw6BBMluZ2hVIZ2033hXSMLEpsc
qmQ5Y7M6MmO5gY0bqsNJf1i+00oP+ioQoqJ7MUm3hKhCRtk+0G1bJokSV3DtTUP5
rrf0uXfHPAsyQJF2Exbnyqy6V4GR3IJJTvNjxDAzxlnnH0RmtkdK+WZb8XwQ20YA
OzLE6XLbaTZSE+YthamfooMruQzkoeDaBWLCGwUOLiJpHVyNZj1sVZTlS7qfGGCr
SL22gwITFbfDYLyCmnlVbu4LR2KXWmpaBlXXJaUh8QKBgQDhLaSadZEyKkmf2IjL
/He4om7fgReC2OoTj3wUNRgJHYIrIzkcdGZDWxgLQodBvfOkM6SqDGJmNf23T7qI
ZBchkuAMGPFv2eRnYApjwKzH4f1ecQ0U0gJO30nCXBM8OLCXD4woQj5GoSeXlhz5
zyayScbmQXPt7sR9rjjw8TEm3wKBgQDa9ouiz54w7xq+8b0AZ4oc19FWoHqP7SYq
lLWDzPTn+4ybpNcscJi6J7KKqZiaKNLI7iviVmLT6NLVDjf9Lg2zcTnbpY8r9B8w
49xbj7kx5Vvt1+J9sqD8QbZHMSe2wgf5yNlsRr87SrewVwvvE7ad2KFgArlp0/6d
LiBkOd6niQKBgFcVMMAvUWymH/z6X8ULqT01TE2RfgczXiscZW7nLZlw2QNXxuFz
Po0z8HOCUg6hqFTLSBYfXfqLTMiUw921X6CzTYRALTcFfChiYwI65FcU1citTdLM
eOoJvlu1AhdbESgKcjirjawA7O/ZtPEDJML0d0Ba9buBiGnWc9zyWgDfAoGAejS5
8E9R6du5ILLImo4vDjQBmQiN/wALmh5PRFVCpqrFaiTRFvNsuhDn2+4VxoxcQFp1
UaiHFeBOsyxxYTOv3+OkuAsp4g0oz1+NH+kSIl/xM8iWlzL4GHIQaqFrmdunGejY
OE8v8cacyKV8ep2VAXnjbzN2CjOQWmdhGq8VrokCgYBbdlnkZkmCkOfZM435u4gj
hibAejAUEW/uPKIjwhkCdWLby6FG5Ve1m3nMbH2xAE7Za8CmMwOCcNe9AKvxS2Fs
BnlzYWG4Ipy8HaUPpRBOwRqF6cohCZ4SovOnx8lFUpOrzlYMd0eYV3BI229xbYyj
zO5gelShWMGm2Bosfcpkzg==
-----END PRIVATE KEY-----",
    );
    set_var(
        "ACCESS_TOKEN_PUBLIC_KEY",
        "-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAwJm62rJrsyHP2MNhKBDG
U520HwoAi3dSygeLcFvDgCy7MucuKc4gjK516yX8YrGP7i+geTolDiLD1B+/WAEc
og8jyHaIlZIHybRouSctj1av+xI924SPxs9E00/5qYmkoDQKA3nnS2J83ncxINIw
WhISwGRDwe//cWtv+GBwBdHA/roQnidOkD4y+WMP91E26vukSSCxiW8ShZMHZ3jV
TGR2HPymoofRJnpj2sU6DQs/QDMM1OH6dHJZI/eGYt8aZZwDko78XQpAafO4cx6d
Dze42KoTcGFZ9ieLvDyqU/mlBQcSTiKRg72elDSw1st8moQmr9gf8xkLJ8+rgAtG
VwIDAQAB
-----END PUBLIC KEY-----",
    );
    set_var(
        "REFRESH_TOKEN_PRIVATE_KEY",
        "-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDDapwKzIF6fGpw
HOerTi7OMZ1s+rvS1Sv96qk4APfYoOkQIebWuCsyBh/Nt+FLN6+Hx7yQ8hjRBPiG
8RR8mTUtY08TDu8+SJVo4mT4BDIO+kzjEFrHUeZgvO1TfqrcPQJ6nKYVeVk+mfB3
8TQUjvlKag2wNE8AtYlYsyWAC7UwxOfukikwEHSAnWNEXGaFJ4mrQAPcfcZMGDWt
MHx3Jz0TIzTOe59LEqs1ZKWFs6U8UcRALie4bR0nfFEXUTV8VN7+IpoEmVcwZo3g
Kz2BZacmkxVqFtF/qmCvjJpQj2pktWcULxJDDJjfvEMYZHx+uRJ3UYR313RZskXR
aK51xdEJAgMBAAECggEAaVcOxsN3CJAI0GbEe5Opp30XX6fJl91R2Y6lqYrcD+qt
uASOazDcGBs1CbAVwnZCKO+Ctp/KwOHtFtDeOkxcXhsqhRuH3AtEf0WLKCca1PgY
ek5WoRuFFKDDTj75278HlxDadrjzYsuY1Q8xA+QOfg6tUk7gR4GUiDwJ+vxUr/SD
7EIK/nTIyBKlpAng9Nf1M0bqXDAIZCZbXwqhIRE43yisNv6ySXJIuYP44Put6FiT
fAAiO93UUxVNr8D9ygXR5u2k8lfYX5CuxHU0caZVNnlVImRoe6sXu3D3O8PWdvi6
j99sXNGq1IuvZAt4jVYCBmA7Uwj+JTAciCVGSQ8oAQKBgQDhcb5lStABMOQw+NvX
ElTbo52laSsd/D9UILbeiIzXEOEhliUgr0Xs6a/8BWimSVLXe5K5wyn3TTpFGDWb
8B1owlzuJbDAXHEUEp95HAQBLC6hg0PBdXQHNQ8HgDhJFCGPHmEpekAm/ed6txcw
N48VkEZpK48JLuZWyCqgrGxlkwKBgQDd5vyj5FNpmFYVJSD86DUoYhjg+o1KfAe5
oCRISDWSpMwyECSSfdqGU39taEKKSEg6/uZXQyzFCMwDJfU9pE5yqbBAgi2c61BF
PnHY++l7kQL6MYm1X2deTPpsdfD4E7a7JOueJRpNbmvxT0CRqddTa+WHcr6OFDNX
cKZMDzkQcwKBgGq6YHqvqj2GGeGdTuZIxWed09olKcZuTsTTH//GAXcnhI1T+Yu5
ro70Kt5S6TIf8FoXJGVRIaL0KqvfRDHowOOBcGFF8qF+ogHwtxKs0rCDbCgGbqM9
qYpn8g+JAhyGrUSGC1WJjKlo9pc/6nhnNRPuU4cimfqs+1sGNDgQqNiFAoGAU0o7
M+0k3fK/BOrNygACy+ieO/vvgrCxWZxhYfwwl51hw7zJbW7t0hOPNeq3Zx4gdnxs
yv481Kwrn1blCUEd7O4CaQQzyL2OgJ8irnZHugjBHFXxwXFQ5FVyfbl7PDJyGSaP
hemZSM0UmRvNOZ+NYhOSsNeB7ORqLPzsfooMOj8CgYEA2KzQLcYn0MoOmaKj0q8d
TErxOrqpwj4I6gfDWM4s047wiU33yeEec6m3nYj2COclvxxwcpzWCuKo0tr3Pgbt
ki3Rbcowh87yE9jHp1EjlABVJz6lwzTbIoi8YnenDu8EriUWdii5MVekJ6oOVou+
CU6/yZQfCLbZn0Amu2b1ldg=
-----END PRIVATE KEY-----",
    );
    set_var(
        "REFRESH_TOKEN_PUBLIC_KEY",
        "-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAw2qcCsyBenxqcBznq04u
zjGdbPq70tUr/eqpOAD32KDpECHm1rgrMgYfzbfhSzevh8e8kPIY0QT4hvEUfJk1
LWNPEw7vPkiVaOJk+AQyDvpM4xBax1HmYLztU36q3D0CepymFXlZPpnwd/E0FI75
SmoNsDRPALWJWLMlgAu1MMTn7pIpMBB0gJ1jRFxmhSeJq0AD3H3GTBg1rTB8dyc9
EyM0znufSxKrNWSlhbOlPFHEQC4nuG0dJ3xRF1E1fFTe/iKaBJlXMGaN4Cs9gWWn
JpMVahbRf6pgr4yaUI9qZLVnFC8SQwyY37xDGGR8frkSd1GEd9d0WbJF0WiudcXR
CQIDAQAB
-----END PUBLIC KEY-----",
    )
}
