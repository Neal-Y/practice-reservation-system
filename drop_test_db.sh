#!/bin/bash
DATABASES=$(psql -U shin -d reservation -t -c "SELECT datname FROM pg_database WHERE datname LIKE 'test_service_%'")

echo "Found databases:"
echo "$DATABASES"

for db in $DATABASES; do
    read -p "Do you want to drop $db? [y/N]: " yn
    case $yn in
        [Yy]* ) psql -U shin -d reservation -c "DROP DATABASE IF EXISTS \"$db\"";;
        * ) echo "Skipped $db";;
    esac
done

#TODO:TBL
