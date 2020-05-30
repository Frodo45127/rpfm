#include "q_list_of_q_standard_item.h"

// Function to be called from any other language. This appends the provided item to the provided QList.
extern "C" void add_to_q_list(QList<QStandardItem*>* list, QStandardItem* item) {
    list->append(item);
}
