#include "cliente.h"
#include <string.h> // memcpy

// this is somehow more awful than the Rust version

typedef struct node {
    Cliente val;
    FILE *source;
    struct node *parent; // :^)
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
    node->left = node->right = node->parent = NULL;

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

    left->parent = right->parent = ret;

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
        for (size_t i = n - 1; i < n; i -= 2) {
            Node *right = iteration[i];
            Node *parent = right;

            if (i > 0) {
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

Node *replace_root(Node *root, Cliente *new_val)
{
    Node *node = root;
    bool is_leaf = false;

    while (!is_leaf) {
        Node *next = NULL;

        if (node->left) {
            if (node->left->source == node->source) {
                next = node->left;
            }
        }

        if (node->right) {
            if (node->right->source == node->source) {
                next = node->right;
            }
        }

        if (!next) {
            is_leaf = true;
        } else {
            node = next;
        }
    }

    if (new_val) {
        node->val = *new_val;

        while (node->parent) {
            node = node->parent;
            if (node->left->val.cod < node->right->val.cod) {
                node->val = node->left->val;
                node->source = node->left->source;
            } else {
                node->val = node->right->val;
                node->source = node->right->source;
            }
        }
    } else {
        if (!node->parent) {
            // fuck C
            free(root);
            return NULL;
        }

        Node *parent = node->parent;

        if (parent->left && parent->left != node) {
            Node *cpy = parent->left;

            parent->val = cpy->val;
            parent->source = cpy->source;

            parent->right = cpy->right;
            parent->left = cpy->left;

            if (cpy->right) {
                cpy->right->parent = parent;
            }

            if (cpy->left) {
                cpy->left->parent = parent;
            }

            free(cpy);
        } else if (parent->right){
            Node *cpy = parent->right;

            parent->val = cpy->val;
            parent->source = cpy->source;

            parent->right = cpy->right;
            parent->left = cpy->left;

            if (cpy->right) {
                cpy->right->parent = parent;
            }

            if (cpy->left) {
                cpy->left->parent = parent;
            }

            free(cpy);
        }

        free(node);

        node = parent;

        while (node->parent) {
            node = node->parent;
            if (node->left->val.cod < node->right->val.cod) {
                node->val = node->left->val;
                node->source = node->left->source;
            } else {
                node->val = node->right->val;
                node->source = node->right->source;
            }
        }
    }

    return node;
}

static size_t sat_sub(size_t x, size_t y)
{
    size_t z = x - y;

    if (z > x) { /* overflow */
        return 0;
    }

    return z;
}

static FILE **load_files(char **files, size_t n)
{
    FILE **ret = malloc(sizeof(FILE *) * n);

    for (size_t i = 0; i < n; ++i) {
        FILE *fp = fopen(files[i], "rb");

        if (!fp) abort();

        ret[i] = fp;
    }

    return ret;
}

static cliente_wrapper *load_entries(FILE **files, size_t n)
{
    cliente_wrapper *ret = malloc(sizeof(cliente_wrapper) * n);

    for (size_t i = 0; i < n; ++i) {
        bool success = cliente_deserialize(&ret[i].cliente, files[i]);

        if (!success) abort();

        ret[i].source = files[i];
    }

    return ret;
}

void nway_merge(char **filenames, size_t files_size, size_t n)
{
    size_t iteration = 0;

    // FUCK C
    char **arena = malloc(sizeof(char *) * files_size);

    while (files_size > 1) {
        char *filename = arena[iteration] = malloc(64);

        snprintf(filename, 64, "merge-%zu", iteration);
        FILE *output = fopen(filename, "wb");

        size_t cutoff = sat_sub(files_size, n - 1);

        size_t cutoff_size = files_size - cutoff;

        FILE **files = load_files(filenames + cutoff, cutoff_size);

        cliente_wrapper *entries = load_entries(files, cutoff_size);

        Node *winner = build_tree(entries, files_size - cutoff);

        while (true) {
            cliente_serialize(&winner->val, output);
            Cliente insert;

            bool success = cliente_deserialize(&insert, winner->source);

            if (!success) {
                winner = replace_root(winner, NULL);
            } else {
                winner = replace_root(winner, &insert);
            }

            if (!winner) {
                break;
            }
        }

        free(entries);
        fclose(output);
        for (size_t i = 0; i < files_size - cutoff; i++) {
            fclose(files[i]);
        }
        free(files);

        files_size = files_size - cutoff_size + 1;
        filenames[files_size - 1] = filename;

        iteration++;
    }

    for (size_t i = 0; i < iteration; ++i) {
        free(arena[i]);
    }

    free(arena);
}

int main(void)
{
    char *files[] = {
        "bucket-0",
        "bucket-1",
        "bucket-2",
        "bucket-3",
        "bucket-4",
        "bucket-5",
        "bucket-6",
        "bucket-7",
        "bucket-8",
        "bucket-9",
        "bucket-10",
        "bucket-11",
        "bucket-12",
        "bucket-13",
        "bucket-14",
        "bucket-15",
        "bucket-16",
        "bucket-17",
        "bucket-18",
        "bucket-19",
        "bucket-20",
        "bucket-21",
        "bucket-22",
        "bucket-23",
        "bucket-24",
    };

    nway_merge(files, 25, 4);

    return 0;
}

