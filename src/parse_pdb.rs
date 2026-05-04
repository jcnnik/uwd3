use std::io::Cursor;

use pdb::FallibleIterator;

pub fn parse_pdb(pdbfile: Vec<u8>) -> u32 {
    let pdbreader = Cursor::new(pdbfile);
    let mut shell32 = pdb::PDB::open(pdbreader).unwrap();
    let symbol_table = shell32.global_symbols().unwrap();
    let address_map = shell32.address_map().unwrap();
    for symbol in symbol_table.iter().iterator().flatten() {
        let data = symbol.parse().unwrap();
        if let pdb::SymbolData::Public(d) = data {
            if d.name.to_string().contains("s_DesktopBuildPaint") && d.function {
                let rva = d.offset.to_rva(&address_map).unwrap();
                return rva.0;
            }
        }
    }
    panic!("Cannot find CDesktopWatermark::s_DesktopBuildPaint in PDB");
}
