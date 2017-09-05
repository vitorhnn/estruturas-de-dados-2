#include <stdio.h>
#include <stdbool.h>
#include "agencia.h"

#define M 7

typedef struct {
    size_t val;
    bool success;
} maybe_size_t;

typedef struct {
    Agencia agencia;
    bool valid;
    bool frozen;
} agencia_wrapper;

static maybe_size_t min(agencia_wrapper *array, size_t n)
{
    ssize_t idx = -1;
    for (size_t i = 0; i < n; ++i) {
        if (!array[i].frozen && array[i].valid) {
            idx = i;
        }
    }

    if (idx == -1) {
        maybe_size_t ret = {
            0,
            false
        };

        return ret;
    }

    // iterate and find the smallest number. ignore all frozen numbers and invalid numbers.
    for (size_t i = 0; i < n; ++i) {
        if (!array[i].frozen &&
                array[i].valid &&
                array[i].agencia.cod < array[idx].agencia.cod)
        {
            idx = i;
        }
    }

    maybe_size_t ret = {
        idx,
        true
    };

    return ret;
}

static bool is_replacement_done(agencia_wrapper *array, size_t n)
{
    bool ret = true;

    for (size_t i = 0; i < n; ++i) {
        if (array[i].valid) {
            ret = false;
        }
    }

    return ret;
}

void replacement(FILE *input)
{
    agencia_wrapper array[M] = {0}; // make valgrind happy
    for (size_t i = 0; i < M; i++) {
        array[i].valid = agencia_deserialize(input, &array[i].agencia);
    }

    size_t bucketN = 0;

    while (!is_replacement_done(array, M)) {
        char name[32];
        sprintf(name, "bucket-%zu", bucketN);
        FILE *bucket = fopen(name, "wb");

        for (maybe_size_t maybeMinIdx = min(array, M);
                maybeMinIdx.success;
                maybeMinIdx = min(array, M))
        {
            size_t minIdx = maybeMinIdx.val;

            agencia_serialize(&array[minIdx].agencia, bucket);

            Agencia justWritten = array[minIdx].agencia;

            array[minIdx].valid = agencia_deserialize(input, &array[minIdx].agencia);

            if (array[minIdx].agencia.cod < justWritten.cod) {
                array[minIdx].frozen = true;
            }
        }

        fclose(bucket);
        bucketN++;

        for (size_t i = 0; i < M; ++i) {
            array[i].frozen = false;
        }
    }
}

int main(void)
{
    FILE *fp = fopen("db", "rb");

    replacement(fp);

    fclose(fp);

    return 0;
}

