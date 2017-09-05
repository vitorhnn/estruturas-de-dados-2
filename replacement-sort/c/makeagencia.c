#include "agencia.h"
#include <stdio.h>
#include <string.h>

#define N 10

// Gera N entradas em ordem inversa.

int main(void)
{
    FILE *fp = fopen("db", "wb");

    for (size_t i = N; i <= N; --i) {
        Agencia a;
        a.cod = i;
        strcpy(a.nome, "agencia aaa");
        a.cod_gerente = i;

        agencia_serialize(&a, fp);
    }

    fclose(fp);
    
    return 0;
}

