cargo install sea-orm-cli
sea-orm-cli generate entity \
    -u mysql://root:3cgpCkSlSMX2EFOu@jira.stargrid.org:45894/remote \
    -o ../models


# CREATE TABLE p2p
# (
#     address VARCHAR(255) PRIMARY KEY,
#     peer_id VARCHAR(255) NOT NULL, 
#     multi_addr VARCHAR(255) NOT NULL
# );