#include "hash_table.h"
#include "endian.h"

typedef struct {
    Cliente val;
    uint64_t key;
    long next;
    bool valid;
} Record;

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
    uint32_t write = htobe32(record->valid);
    fwrite(&next, sizeof(write), 1, out);
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

    read = fread(&out->next, sizeof(out->key), 1, in);

    if (read < 1) {
        return false;
    }

    read = fread(&out->valid, sizeof(int32_t), 1, in);

    if (read < 1) {
        return false;
    }

    out->key = be64toh(out->key);
    out->next = be64toh(out->next);
    out->valid = be32toh(out->valid);

    return true;
}

FileHashTable table_open(FILE *table, FILE *entries, size_t size)
{
    FileHashTable ret;

    ret.table = table;
    ret.entries = entries;
    ret.size = size;

    return ret;
}

FileHashTable table_new(FILE *table, FILE *entries, size_t size)
{
    int64_t minusone = htobe64(-1);

    for (size_t i = 0; i < size; ++i) {
        fwrite(&minusone, sizeof(minusone), 1, table);
    }

    return table_open(table, entries, size);
}

static long seek_tell(FILE *stream, long offset, int whence)
{
    fseek(stream, offset, whence);
    return ftell(stream);
}

static long search_for_empty(FileHashTable *self)
{
    Record record;
    long offset;
    fseek(self->entries, 0, SEEK_SET);
    do {
        offset = ftell(self->entries);
        bool success = record_deserialize(&record, self->entries);

        if (!success) {
            return seek_tell(self->entries, 0, SEEK_END);
        }
    } while (record.valid);

    return offset;
}

void table_insert(FileHashTable *self, uint64_t key, Cliente val)
{
    uint64_t hash = key % self->size; // again, not a real hash function
    uint64_t pos = hash * sizeof(uint64_t);
    fseek(self->table, pos, SEEK_SET);
    int64_t maybe_offset;
    fread(&maybe_offset, sizeof(maybe_offset), 1, self->table);
    maybe_offset = be64toh(maybe_offset);

    if (maybe_offset == -1) {
        long written_at = search_for_empty(self);

        fseek(self->entries, written_at, SEEK_SET);

        Record record = from_cliente(val, key);
        record_serialize(&record, self->entries);
        fseek(self->table, pos, SEEK_SET);
        uint64_t tmp = htobe64(written_at);
        fwrite(&tmp, sizeof(tmp), 1, self->table);
    } else {
        long entry_offset = seek_tell(self->entries, maybe_offset, SEEK_SET);
        Record record;
        record_deserialize(&record, self->entries);

        while (record.next != -1 && record.key != key) {
            entry_offset = seek_tell(self->entries, record.next, SEEK_SET);
            record_deserialize(&record, self->entries);
        }

        if (record.key == key) {
            // already present
            return;
        }

        long written_at = search_for_empty(self);
        fseek(self->entries, written_at, SEEK_SET);

        Record new_record = from_cliente(val, key);
        record_serialize(&new_record, self->entries);
        record.next = written_at;
        fseek(self->entries, entry_offset, SEEK_SET);
        record_serialize(&record, self->entries);
    }
}
