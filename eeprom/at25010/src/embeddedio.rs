use embedded_hal_async::{delay, spi};
use embedded_io::{asynch, Error, ErrorKind, Io, SeekFrom};

use crate::{driver::StatefulDriver, DriverError};

impl Error for DriverError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

impl<Spi, Delay> Io for StatefulDriver<Spi, Delay>
where
    Spi: spi::SpiDevice,
    Delay: delay::DelayUs,
{
    type Error = DriverError;
}

impl<Spi, Delay> asynch::Seek for StatefulDriver<Spi, Delay>
where
    Spi: spi::SpiDevice,
    Delay: delay::DelayUs,
{
    async fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        let pos = match pos {
            SeekFrom::Start(offset) => offset as i64,
            SeekFrom::End(offset) => self.driver.capacity() as i64 + offset,
            SeekFrom::Current(offset) => self.position as i64 + offset,
        };

        assert!(pos >= 0);
        let pos = pos as u64;
        if pos > self.driver.capacity() as u64 {
            return Err(DriverError::Capacity);
        }

        self.position = pos as u16;
        Ok(pos)
    }
}

impl<Spi, Delay> asynch::Read for StatefulDriver<Spi, Delay>
where
    Spi: spi::SpiDevice,
    Delay: delay::DelayUs,
{
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let length = usize::min(
            self.position as usize + buf.len(),
            self.driver.capacity() as usize,
        );
        self.driver.read(self.position, &mut buf[..length]).await?;
        self.position += length as u16;
        Ok(length)
    }
}

impl<Spi, Delay> asynch::Write for StatefulDriver<Spi, Delay>
where
    Spi: spi::SpiDevice,
    Delay: delay::DelayUs,
{
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let length = usize::min(
            self.position as usize + buf.len(),
            self.driver.capacity() as usize,
        );
        self.driver.write(self.position, &buf[..length]).await?;
        self.position += length as u16;
        Ok(length)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.driver.flush().await
    }
}
