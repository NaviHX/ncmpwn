use super::error::{DumpResult, Error};
use id3::Tag as ID3v2InnerTag;
use metaflac::Tag as FlacInnerTag;

pub trait TagWrite {
    fn write_with_tag_to(&mut self, writer: &mut impl std::io::Write) -> DumpResult<()>;
}

impl TagWrite for ID3v2InnerTag {
    fn write_with_tag_to(&mut self, writer: &mut impl std::io::Write) -> DumpResult<()> {
        self.write_to(writer, id3::Version::Id3v24)
            .map_err(|e| Error::TagWritedError(e.to_string()))
    }
}

impl TagWrite for FlacInnerTag {
    fn write_with_tag_to(&mut self, writer: &mut impl std::io::Write) -> DumpResult<()> {
        self.write_to(writer)
            .map_err(|e| Error::TagWritedError(e.to_string()))
    }
}
