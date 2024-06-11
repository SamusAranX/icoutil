use ico::IconDirEntry;

use crate::ICON_SIZES;

pub fn construct_entry_name(entry: &IconDirEntry) -> String {
	let mut file_name = format!("{}", entry.width());
	if entry.width() != entry.height() || !ICON_SIZES.contains(&entry.width()) {
		file_name.push_str(format!("x{}", entry.height()).as_str())
	}
	if entry.bits_per_pixel() != 32 {
		file_name.push_str(format!("@{}", entry.bits_per_pixel()).as_str());
	}
	file_name.push_str(".png");
	file_name
}