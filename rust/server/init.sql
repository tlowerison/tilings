SELECT 'CREATE DATABASE tilings'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'tilings')\gexec
