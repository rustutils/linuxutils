/* SPDX-License-Identifier: LGPL-2.1-or-later */
/*
 * blkid.h — partial ABI-compatible header for linuxutils-libblkid.
 *
 * Hand-crafted v0 subset of upstream util-linux's libblkid API. The
 * upstream autoconf template is preserved alongside as `blkid.h.in`
 * for reference.
 *
 * Scope (v0): the udev-symlink resolution layer — cache, tag/spec
 * resolve, device + tag iteration. The probe layer (anything
 * `blkid_probe_*`, partitions, topology) is not implemented.
 */

#ifndef _LIBBLKID_BLKID_H
#define _LIBBLKID_BLKID_H

#ifdef __cplusplus
extern "C" {
#endif

#include <sys/types.h>
#include <stdint.h>

/* Opaque types. */
typedef struct blkid_struct_dev          *blkid_dev;
typedef struct blkid_struct_cache        *blkid_cache;
typedef struct blkid_struct_dev_iterate  *blkid_dev_iterate;
typedef struct blkid_struct_tag_iterate  *blkid_tag_iterate;

/* Library scaffolding. */
extern void blkid_init_debug(int mask);
extern int blkid_get_library_version(const char **ver_string,
                                     const char **date_string);

/* Cache lifecycle. The `filename` argument is accepted for upstream
 * compatibility but ignored — linuxutils-libblkid resolves via the
 * `/dev/disk/by-*` udev symlinks and does not read or write an
 * on-disk cache file. */
extern int blkid_get_cache(blkid_cache *cache, const char *filename);
extern void blkid_put_cache(blkid_cache cache);

/* Tag / spec resolution. The strings returned by these functions
 * are heap-allocated via malloc(3); the caller must free(3) them. */
extern char *blkid_evaluate_tag(const char *token, const char *value,
                                blkid_cache *cache);
extern char *blkid_evaluate_spec(const char *spec, blkid_cache *cache);
extern char *blkid_get_devname(blkid_cache cache, const char *token,
                               const char *value);
extern char *blkid_get_tag_value(blkid_cache cache, const char *tagname,
                                 const char *devname);

/* Parse "TAG=value" into separately-malloc'd token and value. */
extern int blkid_parse_tag_string(const char *token, char **ret_type,
                                  char **ret_val);

/* Device queries. The returned blkid_dev is owned by the cache and
 * must not be freed by the caller. */
extern blkid_dev blkid_get_dev(blkid_cache cache, const char *devname,
                               int flags);
extern const char *blkid_dev_devname(blkid_dev dev);
extern int blkid_dev_has_tag(blkid_dev dev, const char *type,
                             const char *value);
extern blkid_dev blkid_find_dev_with_tag(blkid_cache cache, const char *type,
                                         const char *value);

/* blkid_get_dev flags. */
#define BLKID_DEV_FIND      0x0000
#define BLKID_DEV_CREATE    0x0001
#define BLKID_DEV_VERIFY    0x0002
#define BLKID_DEV_NORMAL    (BLKID_DEV_CREATE | BLKID_DEV_VERIFY)

/* Iteration. */
extern blkid_dev_iterate blkid_dev_iterate_begin(blkid_cache cache);
extern int blkid_dev_next(blkid_dev_iterate iterate, blkid_dev *dev);
extern void blkid_dev_iterate_end(blkid_dev_iterate iterate);

extern blkid_tag_iterate blkid_tag_iterate_begin(blkid_dev dev);
extern int blkid_tag_next(blkid_tag_iterate iterate, const char **type,
                          const char **value);
extern void blkid_tag_iterate_end(blkid_tag_iterate iterate);

#ifdef __cplusplus
}
#endif

#endif /* _LIBBLKID_BLKID_H */
