use x86::io;

use core::ptr;


#[derive(Debug)]
pub enum AtaError {
    NotFound,
    NotAta,
    Poll,
}

#[non_exhaustive]
pub struct Status;

impl Status {
    // https://wiki.osdev.org/ATA_PIO_Mode#Status_Register_(I/O_base_+_7)

    const BSY: u8 = 0x80;
    const DRQ: u8 = 0x8;
    const ERR: u8 = 0x1;
}

#[non_exhaustive]
pub struct Ata {
    sectors: u32,
}

impl Ata {
    const IDENTIFY: u8 = 0xec;
    const READ: u8 = 0x20;
    const WRITE: u8 = 0x30;
    const FLUSH: u8 = 0xe7;

    const SECTOR_COUNT_REGISTER: u16 = 0x1f2;
    const SECTOR_NUMBER_REGISTER: u16 = 0x1f3;
    const FEATURE_REGISTER: u16 = 0x1f1;
    const DATA_REGISTER: u16 = 0x1f0;
    const HEAD_REGISTER: u16 = 0x1f6;
    const SC_REGISTER: u16 = 0x1f7;
    const LOW_REGISTER: u16 = 0x1f4;
    const HIGH_REGISTER: u16 = 0x1f5;

    pub fn new() -> Ata {
        Ata {
            sectors: 0,
        }
    }

    unsafe fn setup(&self, lba: u32, sector_count: u8) {
        io::outb(Ata::HEAD_REGISTER, (0xe0 | (lba >> 24) & 0x0f) as u8);
        io::outb(Ata::FEATURE_REGISTER, 0x00);
        io::outb(Ata::SECTOR_COUNT_REGISTER, sector_count);

        io::outb(Ata::SECTOR_NUMBER_REGISTER, (lba & 0xff) as u8);
        io::outb(Ata::LOW_REGISTER, ((lba >> 8) & 0xff) as u8);
        io::outb(Ata::HIGH_REGISTER, ((lba >> 16) & 0xff) as u8);
    }

    pub fn write(&self, lba: u32, data: &[u8]) {
        unsafe {
            self.setup(lba, (data.len() / 512) as u8 + 1);

            io::outb(Ata::SC_REGISTER, Ata::WRITE);

            for sector in data.chunks(512) {
                let mut sector = sector.to_vec();

                sector.resize(512, 0);

                self.write_sector(&sector);
            }

            io::outb(Ata::SC_REGISTER, Ata::FLUSH);

            while io::inb(Ata::SC_REGISTER) & Status::BSY != 0 {}
        }
    }

    fn write_sector(&self, sector: &[u8]) {
        unsafe {
            assert_eq!(sector.len(), 512);

            for (low, high) in sector.chunks(2).map(|chunk| ( chunk[0] as u16, chunk[1] as u16 )) {
                io::outw(Ata::DATA_REGISTER, low << 8 | high);
            }
        }
    }

    // lba is basicly just a sector address
    //
    // since one sector is 256 16-bit values the same sector will have the length of 512 if we
    // represent it as 8-bit values
    //
    // out must have a size equal or bigger then sector_count * 512
    pub fn read(&self, lba: u32, sector_count: u8, mut out: *mut u8) -> Result<(), AtaError> {
        unsafe {
            self.setup(lba, sector_count);

            io::outb(Ata::SC_REGISTER, Ata::READ);

            for _ in 0..sector_count {
                while io::inb(Ata::SC_REGISTER) & Status::DRQ != 0 {}

                let sector = self.read_sector()?;

                ptr::copy_nonoverlapping::<u8>(sector.as_ptr() as *const u8, out, 512);

                out = out.byte_offset(512);
            }

            Ok(())
        }
    }

    fn read_sector(&self) -> Result<[u16; 256], AtaError> {
        unsafe {
            let mut buffer: [u16; 256] = [0; 256];

            for value in buffer.iter_mut() {
                *value = io::inw(Ata::DATA_REGISTER);
            }

            Ok(buffer)
        }
    }

    /*
     *
     * To use the IDENTIFY command, select a target drive by sending 0xA0 for the master drive,
     * or 0xB0 for the slave, to the "drive select" IO port. On the Primary bus, this would be port 0x1F6.
     * Then set the Sectorcount, LBAlo, LBAmid, and LBAhi IO ports to 0 (port 0x1F2 to 0x1F5).
     * Then send the IDENTIFY command (0xEC) to the Command IO port (0x1F7).
     * Then read the Status port (0x1F7) again.
     * If the value read is 0, the drive does not exist.
     * For any other value: poll the Status port (0x1F7) until bit 7 (BSY, value = 0x80) clears.
     * Because of some ATAPI drives that do not follow spec,
     * at this point you need to check the LBAmid and LBAhi ports (0x1F4 and 0x1F5) to see if they are non-zero.
     * If so, the drive is not ATA, and you should stop polling.
     * Otherwise, continue polling one of the Status ports until bit 3 (DRQ, value = 8) sets,
     * or until bit 0 (ERR, value = 1) sets.
     *
    */

    pub fn identify(&mut self) -> Result<(), AtaError> {
        // TODO: this function hangs somewhere, it may be related to the software reset thingy

        unsafe {
            io::outb(Ata::HEAD_REGISTER, 0xa0);

            for port in 0x1f2..=0x1f5 {
                io::outb(port, 0);
            }

            io::outb(Ata::SC_REGISTER, Ata::IDENTIFY);

            match io::inb(Ata::SC_REGISTER) {
                0 => return Err(AtaError::NotFound),
                _ => while io::inb(Ata::SC_REGISTER) & Status::BSY != 0 {},
            }

            if io::inb(Ata::LOW_REGISTER) != 0 || io::inb(Ata::HIGH_REGISTER) != 0 {
                return Err(AtaError::NotAta);
            }

            let mut status = io::inb(Ata::SC_REGISTER);

            while status & Status::DRQ == 0 && status & Status::ERR == 0 {
                status = io::inb(Ata::SC_REGISTER);
            }

            if status & Status::ERR != 0 {
                return Err(AtaError::Poll);
            }

            let sector = self.read_sector()?;

            self.sectors = (sector[60] as u32) << 16 | sector[61] as u32;

            Ok(())
        }
    }
}



