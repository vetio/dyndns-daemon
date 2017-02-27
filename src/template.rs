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
            .flat_map(
                |part| part
                    .split("{%IP%}")
                    .map(|part| TemplateSegment::Static(String::from(part)))
                    .intersperse(TemplateSegment::Ip)
            )
            .intersperse(TemplateSegment::Serial)
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

