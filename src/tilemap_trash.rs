

static int stbte__strequal(char *p, char *q)
{
	while (*p)
		if (*p++ != *q++) return 0;
	return *q == 0;
}