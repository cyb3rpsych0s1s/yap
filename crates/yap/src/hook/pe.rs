use libloading::os::windows::Library;
use std::ops::Range;

/// `IMAGE_DOS_HEADER::e_lfanew`: offset of the `IMAGE_NT_HEADERS64`
const IMAGE_DOS_E_LFANEW: usize = 0x3C;
/// `IMAGE_NT_HEADERS64::FileHeader`, after `Signature` (4)
const IMAGE_NT_FILE_HEADER: usize = 0x4;
/// `IMAGE_FILE_HEADER::NumberOfSections`
const IMAGE_FILE_NUMBER_OF_SECTIONS: usize = 0x2;
/// `IMAGE_FILE_HEADER::SizeOfOptionalHeader`
const IMAGE_FILE_SIZE_OF_OPTIONAL_HEADER: usize = 0x10;
/// `IMAGE_NT_HEADERS64::OptionalHeader`, after `Signature` (4) and `IMAGE_FILE_HEADER` (0x14)
const IMAGE_NT_OPTIONAL_HEADER: usize = 0x18;
/// `IMAGE_OPTIONAL_HEADER64::SizeOfImage`
const IMAGE_OPTIONAL_SIZE_OF_IMAGE: usize = 0x38;
/// `sizeof(IMAGE_SECTION_HEADER)`
const IMAGE_SECTION_HEADER_SIZE: usize = 0x28;
/// `IMAGE_SECTION_HEADER::Misc.VirtualSize`, after `Name` (8)
const IMAGE_SECTION_VIRTUAL_SIZE: usize = 0x8;
/// `IMAGE_SECTION_HEADER::VirtualAddress`
const IMAGE_SECTION_VIRTUAL_ADDRESS: usize = 0xC;

/// base address (`HMODULE`) of a module already loaded in the process, e.g. `RED4ext.dll`
pub(super) fn module_base(name: &str) -> Result<usize, libloading::Error> {
    let base = Library::open_already_loaded(name)?.into_raw() as usize;
    // balance the refcount taken by open_already_loaded
    drop(unsafe { Library::from_raw(base as _) });
    Ok(base)
}

unsafe fn read<T: Copy>(addr: usize) -> T {
    unsafe { (addr as *const T).read_unaligned() }
}

fn nt_headers(base: usize) -> usize {
    base + unsafe { read::<u32>(base + IMAGE_DOS_E_LFANEW) } as usize
}

/// address range of the module image in memory, read from its PE headers
pub(super) fn image_range(base: usize) -> Range<usize> {
    let size = unsafe {
        read::<u32>(nt_headers(base) + IMAGE_NT_OPTIONAL_HEADER + IMAGE_OPTIONAL_SIZE_OF_IMAGE)
    } as usize;
    base..base + size
}

/// address range of a section in memory (e.g. `.rdata`), read from the module's section headers
pub(super) fn section(base: usize, name: &[u8]) -> Option<Range<usize>> {
    let file = nt_headers(base) + IMAGE_NT_FILE_HEADER;
    let (count, optional) = unsafe {
        (
            read::<u16>(file + IMAGE_FILE_NUMBER_OF_SECTIONS) as usize,
            read::<u16>(file + IMAGE_FILE_SIZE_OF_OPTIONAL_HEADER) as usize,
        )
    };
    let first = nt_headers(base) + IMAGE_NT_OPTIONAL_HEADER + optional;
    (0..count)
        .map(|idx| first + idx * IMAGE_SECTION_HEADER_SIZE)
        .find(|&header| {
            // NUL-padded to 8 bytes
            let raw = unsafe { read::<[u8; 8]>(header) };
            raw.split(|&b| b == 0).next() == Some(name)
        })
        .map(|header| unsafe {
            let start = base + read::<u32>(header + IMAGE_SECTION_VIRTUAL_ADDRESS) as usize;
            start..start + read::<u32>(header + IMAGE_SECTION_VIRTUAL_SIZE) as usize
        })
}
