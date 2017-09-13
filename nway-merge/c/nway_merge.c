#include "cliente.h"
#include <string.h> // memcpy

typedef struct node {
    Cliente val;
    FILE *source;
    struct node *left;
    struct node *right;
} Node;

typedef struct {
    Cliente cliente;
    FILE *source;
} cliente_wrapper;

static Node *leafify(cliente_wrapper val)
{
    Node *node = malloc(sizeof(Node));

    if (!node) abort();

    node->val = val.cliente;
    node->source = val.source;
    node->left = node->right = NULL;

    return node;
}

Node *choose_winner(Node *left, Node *right)
{
    Node *ret = malloc(sizeof(Node));

    if (left->val.cod < right->val.cod) {
        memcpy(ret, left, sizeof(Node));

        ret->left = left;
        ret->right = right;
    } else {
        memcpy(ret, right, sizeof(Node));

        ret->left = left;
        ret->right = right;
    }

    return ret;
}

Node *build_tree(cliente_wrapper *array, size_t n)
{
    Node **iteration = malloc(sizeof(Node *) * n);
    Node **next_iteration = malloc(sizeof(Node *) * n);

    size_t next_it = 0;

    for (size_t i = 0; i < n; ++i) {
        iteration[i] = leafify(array[i]);
    }

    while (n > 1) {
        for (size_t i = n; i <= n; --i) {
            Node *right = iteration[i];
            Node *parent = right;

            if (i > 1) {
                Node *left = iteration[i - 1];

                parent = choose_winner(left, right);
            }

            next_iteration[next_it] = parent;
            ++next_it;
        }
        n = next_it;
        next_it = 0;
        Node **temp = iteration;
        iteration = next_iteration;
        next_iteration = temp;
    }

    Node *winner = iteration[0];

    free(iteration);
    free(next_iteration);

    return winner;
}

int main(void)
{
    return 0;
}
