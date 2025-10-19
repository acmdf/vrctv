rm *.sqlite

# Replace the members in the Cargo.toml workspace from
# members = ["project-lily-common", "project-lily-server", "project-lily-desktop\\src-tauri"]
# to
# members = ["project-lily-common", "project-lily-server"]

sed -i.bak 's/members = \[.*\]/members = ["project-lily-common", "project-lily-server"]/' Cargo.toml

cargo build --release -p project-lily-server

systemctl restart lily