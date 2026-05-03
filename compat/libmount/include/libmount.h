/* SPDX-License-Identifier: LGPL-2.1-or-later */
/*
 * libmount.h — partial ABI-compatible header for linuxutils-libmount.
 *
 * Hand-crafted v0 subset of upstream util-linux's libmount API. The
 * upstream autoconf template is preserved alongside as
 * `libmount.h.in` for reference; this header declares only the
 * functions actually exported by linuxutils-libmount.
 *
 * Scope (v0): read-only iteration and lookup over fstab, mountinfo,
 * and /proc/swaps. No context, no cache, no lock, no monitor, no
 * setters.
 */

#ifndef _LIBMOUNT_MOUNT_H
#define _LIBMOUNT_MOUNT_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdio.h>
#include <sys/types.h>
#include <stdint.h>

/* Iterator direction. */
#define MNT_ITER_FORWARD   0
#define MNT_ITER_BACKWARD  1

/* Opaque types. */
struct libmnt_fs;
struct libmnt_table;
struct libmnt_iter;

/* Library scaffolding. */
extern int mnt_init_debug(int mask);
extern const char *mnt_get_library_version(const char **ver_string);

/* libmnt_fs — lifecycle. */
extern struct libmnt_fs *mnt_new_fs(void);
extern void mnt_free_fs(struct libmnt_fs *fs);
extern void mnt_ref_fs(struct libmnt_fs *fs);
extern void mnt_unref_fs(struct libmnt_fs *fs);

/* libmnt_fs — getters. */
extern const char *mnt_fs_get_source(struct libmnt_fs *fs);
extern const char *mnt_fs_get_target(struct libmnt_fs *fs);
extern const char *mnt_fs_get_fstype(struct libmnt_fs *fs);
extern const char *mnt_fs_get_options(struct libmnt_fs *fs);
extern const char *mnt_fs_get_vfs_options(struct libmnt_fs *fs);
extern const char *mnt_fs_get_fs_options(struct libmnt_fs *fs);
extern int mnt_fs_get_freq(struct libmnt_fs *fs);
extern int mnt_fs_get_passno(struct libmnt_fs *fs);
extern int mnt_fs_get_id(struct libmnt_fs *fs);
extern int mnt_fs_get_parent_id(struct libmnt_fs *fs);

/* libmnt_table — lifecycle. */
extern struct libmnt_table *mnt_new_table(void);
extern void mnt_free_table(struct libmnt_table *tb);
extern void mnt_ref_table(struct libmnt_table *tb);
extern void mnt_unref_table(struct libmnt_table *tb);

/* libmnt_table — parsing. Each returns 0 on success, -errno on
 * failure. The table is appended to (a fresh table is empty). */
extern int mnt_table_parse_fstab(struct libmnt_table *tb, const char *filename);
extern int mnt_table_parse_mtab(struct libmnt_table *tb, const char *filename);
extern int mnt_table_parse_swaps(struct libmnt_table *tb, const char *filename);

/* libmnt_table — read-only access. */
extern int mnt_table_get_nents(struct libmnt_table *tb);

extern int mnt_table_first_fs(struct libmnt_table *tb, struct libmnt_fs **fs);
extern int mnt_table_next_fs(struct libmnt_table *tb,
                             struct libmnt_iter *itr,
                             struct libmnt_fs **fs);

extern struct libmnt_fs *mnt_table_find_target(struct libmnt_table *tb,
                                               const char *path,
                                               int direction);
extern struct libmnt_fs *mnt_table_find_source(struct libmnt_table *tb,
                                               const char *source,
                                               int direction);

/* libmnt_iter. */
extern struct libmnt_iter *mnt_new_iter(int direction);
extern void mnt_free_iter(struct libmnt_iter *itr);
extern void mnt_reset_iter(struct libmnt_iter *itr, int direction);
extern int mnt_iter_get_direction(struct libmnt_iter *itr);

#ifdef __cplusplus
}
#endif

#endif /* _LIBMOUNT_MOUNT_H */
