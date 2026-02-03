rm *.sqlite

# Replace the members in the Cargo.toml workspace from
# members = [
#     "vrctv-common",
#     "vrctv-server",
#     "vrctv-desktop/src-tauri",
#     "vrctv-overlay",
# ]
# to
# members = ["vrctv-common", "vrctv-server"]
sed -i 's/members = \[/members = [/; /members = \[/,/\]/s/"vrctv-desktop\/src-tauri",\s*//; /members = \[/,/\]/s/"vrctv-overlay",\s*//' Cargo.toml

cargo build --release -p vrctv-server

systemctl restart vrctv.service