use errors::*;

use std::net;

use chrono::*;

#[derive(Clone, Debug)]
enum TemplateSegment {
    Static(String),
    Ip,
    Serial,
}

#[derive(Clone, Debug)]
pub struct Template {
    segments: Vec<TemplateSegment>,
}

impl Template {
    pub fn from_str(template: &str) -> Self {
        use itertools::Itertools;

        let segments = template
            .split("{%SERIAL%}")
            .map(
                |part| part
                    .split("{%IP%}")
                    .map(|part| TemplateSegment::Static(String::from(part)))
                    .intersperse(TemplateSegment::Ip)
                    .collect()
            )
            .intersperse(vec![TemplateSegment::Serial])
            .flat_map(|s| s)
            .collect();

        Template {
            segments: segments,
        }
    }

    pub fn render(&self, ip: &net::Ipv4Addr, now: DateTime<UTC>) -> Result<String> {
        const MAX_IP_SIZE: usize = 15;
        const MAX_SERIAL_SIZE: usize = 19;

        let mut size = 0;

        for segment in &self.segments {
            match segment {
                &TemplateSegment::Ip => size += MAX_IP_SIZE,
                &TemplateSegment::Serial => size += MAX_SERIAL_SIZE,
                &TemplateSegment::Static(ref s) => size += s.len(),
            }
        }

        let mut buffer = String::with_capacity(size);

        for segment in &self.segments {
            use std::fmt::Write;

            match segment {
                &TemplateSegment::Ip => write!(buffer, "{}", ip)
                    .chain_err(|| "Error formatting ip address")?,
                &TemplateSegment::Serial => write!(buffer, "{}", now)
                    .chain_err(|| "Error formatting serial")?,
                &TemplateSegment::Static(ref s) => buffer += &s,
            };
        }

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono;
    use quickcheck;
    use std::net;

    #[derive(Clone, Copy, Debug)]
    struct Time(chrono::DateTime<chrono::UTC>);

    impl quickcheck::Arbitrary for Time {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            let seconds: i32 = g.gen();
            let time = chrono::NaiveDateTime::from_timestamp(seconds as i64, 0);

            Time(chrono::DateTime::from_utc(time, chrono::UTC))
        }
    }

    #[derive(Clone, Copy, Debug)]
    struct Ip(net::Ipv4Addr);

    impl quickcheck::Arbitrary for Ip {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            Ip(net::Ipv4Addr::from(g.gen::<u32>()))
        }
    }

    impl quickcheck::Arbitrary for TemplateSegment {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            match g.choose(&[TemplateSegment::Serial, TemplateSegment::Ip, TemplateSegment::Static("".into())]) {
                Some(s @ &TemplateSegment::Ip) | Some(s @ &TemplateSegment::Serial) => s.clone(),
                Some(&TemplateSegment::Static(_)) => TemplateSegment::Static(String::arbitrary(g)),
                None => unreachable!(),
            }
        }
    }

    #[quickcheck]
    fn segments_append_to_text(ip: Ip, now: Time, segments: Vec<TemplateSegment>) -> Result<bool> {
        let mut expected = String::new();
        let mut template = String::new();

        for s in &segments {
            use std::fmt::Write;
            match s {
                &TemplateSegment::Serial => {
                    write!(expected, "{}", now.0)
                        .chain_err(|| "Error writing time")?;
                    template += "{%SERIAL%}";
                },
                &TemplateSegment::Ip => {
                    write!(expected, "{}", ip.0)
                        .chain_err(|| "Error writing ip")?;
                    template += "{%IP%}";
                },
                &TemplateSegment::Static(ref s) => {
                    expected += s;
                    template += s;
                },
            };
        }

        let template = Template::from_str(&template);

        let result = template.render(&ip.0, now.0)?;

        Ok(result == expected)
    }
}
