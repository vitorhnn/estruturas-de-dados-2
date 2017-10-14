#include "hash_table.h"

typedef struct {
    Cliente val;
    uint64_t key;
    int64_t next;
    bool valid;
} Record;

static size_t fuck_this_horrible_language(void)
{
    return sizeof(uint32_t) +
        sizeof(char) * 100 + // always 100
        sizeof(uint64_t) +
        sizeof(int64_t) +
        sizeof(bool);
}

static Record from_cliente(Cliente cliente, uint64_t key)
{
    Record ret;

    ret.val = cliente;
    ret.key = key;
    ret.next = -1;
    ret.valid = true;

    return ret;
}

static void record_serialize(Record *record, FILE *out)
{
    cliente_serialize(&record->val, out);
    uint64_t key = htobe64(record->key);
    fwrite(&key, sizeof(key), 1, out);

    int64_t next = htobe64(record->next);
    fwrite(&next, sizeof(next), 1, out);

    uint32_t write = record->valid ? 1 : 0;
    write = htobe32(write);
    fwrite(&write, sizeof(write), 1, out);
}

static bool record_deserialize(Record *out, FILE *in)
{
    if (!cliente_deserialize(&out->val, in)) {
        return false;
    }

    size_t read = fread(&out->key, sizeof(out->key), 1, in);
    if (read < 1) {
        return false;
    }

    read = fread(&out->next, sizeof(out->next), 1, in);

    if (read < 1) {
        return false;
    }

    uint32_t tmp;

    read = fread(&tmp, sizeof(tmp), 1, in);

    if (read < 1) {
        return false;
    }

    out->key = be64toh(out->key);
    out->next = be64toh(out->next);
    out->valid = be32toh(tmp);

    return true;
}

static long seek_tell(FILE *stream, long offset, int whence)
{
    fseek(stream, offset, whence);
    return ftell(stream);
}

FileHashTable table_open(FILE *table, size_t size)
{
    FileHashTable ret;

    ret.table = table;
    ret.size = size;

    return ret;
}

FileHashTable table_new(FILE *table, size_t size)
{
    uint8_t dummy = 0;

    size_t sz = sizeof(FILE *) + sizeof(size_t);

    for (size_t i = 0; i < sz * size; ++i) {
        fwrite(&dummy, sizeof(dummy), 1, table);
    }

    return table_open(table, size);
}

static long search_for_empty(FileHashTable *self)
{
    Record record;
    long offset;
    fseek(self->table, 0, SEEK_SET);

    do {
        offset = ftell(self->table);
        bool success = record_deserialize(&record, self->table);

        if (!success) {
            return seek_tell(self->table, 0, SEEK_SET);
        }
    } while (record.valid);

    return offset;
}

void table_insert(FileHashTable *self, uint64_t key, Cliente val)
{
    uint64_t hash = key % self->size;
    uint64_t pos = hash * fuck_this_horrible_language();
    fseek(self->table, pos, SEEK_SET);

    Record entry;
    record_deserialize(&entry, self->table);

    if (!entry.valid) {
        fseek(self->table, pos, SEEK_SET);

        Record new_entry = from_cliente(val, key);
        record_serialize(&new_entry, self->table);
    } else {
        uint64_t prev_offset = pos;
        while (entry.next != -1) {
            prev_offset = seek_tell(self->table, entry.next, SEEK_SET);
            record_deserialize(&entry, self->table);
        }

        long write_at = search_for_empty(self);
        fseek(self->table, write_at, SEEK_SET);
        Record write = from_cliente(val, key);
        record_serialize(&write, self->table);
        fseek(self->table, prev_offset, SEEK_SET);
        entry.next = write_at;
        record_serialize(&entry, self->table);
    }
}

void table_delete(FileHashTable *self, uint64_t key)
{
    uint64_t hash = key % self->size;
    uint64_t pos = hash * fuck_this_horrible_language();
    fseek(self->table, pos, SEEK_SET);

    Record entry;
    record_deserialize(&entry, self->table);

    if (!entry.valid) {
        return;
    } else {
        Record prev = entry;
        long prev_offset = pos, cur_offset = pos;
        while (entry.key != key && entry.next != -1) {
            prev = entry;
            prev_offset = cur_offset;
            cur_offset = seek_tell(self->table, entry.next, SEEK_SET);
            record_deserialize(&entry, self->table);
        }

        if (entry.key != key) {
            return;
        }


        if (prev_offset != cur_offset) { // not the first element in the list
            entry.valid = false;
            entry.next = -1;
            fseek(self->table, cur_offset, SEEK_SET);
            record_serialize(&entry, self->table);
            fseek(self->table, prev_offset, SEEK_SET);
            prev.next = -1;
            record_serialize(&prev, self->table);
        } else { // first entry in list. move the second to here, if it exists.
            if (entry.next != -1) {
                fseek(self->table, entry.next, SEEK_SET);
                Record next;
                record_deserialize(&next, self->table);
                fseek(self->table, cur_offset, SEEK_SET);
                record_serialize(&next, self->table);
            }
        }
    }
}

Cliente table_search(FileHashTable *self, uint64_t key)
{
    uint64_t hash = key % self->size;
    uint64_t pos = hash * fuck_this_horrible_language();
    fseek(self->table, pos, SEEK_SET);

    Record entry;
    record_deserialize(&entry, self->table);

    if (!entry.valid) {
        abort();
    } else {
        while (entry.key != key && entry.next != -1) {
            fseek(self->table, entry.next, SEEK_SET);
            record_deserialize(&entry, self->table);
        }

        if (entry.key != key) {
            abort();
        }

        return entry.val;
    }
}
