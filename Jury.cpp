#include <bits/stdc++.h>
using namespace std;
int main(const int argc, const char **argv) {
#ifdef RINNE
    cout << argc << endl;
    for (int i = 0; i < argc; i++) {
        cout << argv[i] << endl;
    }
#endif
    if (argc != 4) {
#ifdef RINNE
        cout << "GUALE" << endl;
#endif
        return -1;
    }
    const char *input = argv[1];
    const char *output = argv[2];
    const char *answer = argv[3];
    ifstream my(output);
    ifstream std(answer);
    string s1, s2;
    while (my >> s1) {
        if (std >> s2 && s1 == s2) {
            continue;
        } else {
            return 1;
        }
    }
    return 0;
}
