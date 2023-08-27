#ifndef PHP_RS_WRAPPER_H
#define PHP_RS_WRAPPER_H
#include <Zend/zend.h>
#include <Zend/zend_string.h>
#include <main/php.h>
#include <sapi/embed/php_embed.h>

zend_string *_zend_string_init(const char *str, size_t len, bool persistent)
{
	return zend_string_init(str, len, persistent);
}

void _zend_string_release(zend_string *s)
{
    zend_string_release(s);
}

void _zend_string_release_ex(zend_string *s, bool persistent)
{
    zend_string_release_ex(s, persistent);
}

zend_string *_zend_string_realloc(zend_string *s, size_t len, bool persistent)
{
    return zend_string_realloc(s, len, persistent);
}

zend_string *_zend_string_extend(zend_string *s, size_t len, bool persistent)
{
    return zend_string_extend(s, len, persistent);
}

zend_string *_zend_string_truncate(zend_string *s, size_t len, bool persistent)
{
    return zend_string_truncate(s, len, persistent);
}

zend_string *_zend_string_dup(zend_string *s, bool persistent)
{
    return zend_string_dup(s, persistent);
}
#endif
