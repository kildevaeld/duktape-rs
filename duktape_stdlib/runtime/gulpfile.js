const gulp = require('gulp'),
    uglify = require('gulp-uglify'),
    babel = require('gulp-babel'),
    minify = require("gulp-babel-minify"),
    replace = require('gulp-replace');

gulp.task('uglify', () => {
    return gulp.src('./src/**/*.js')
        .pipe(babel())
        .pipe(minify({
            mangle: {
                keepClassName: true
            }
        }))
        //.pipe(babel())
        .pipe(replace('"use strict";', ''))
        .pipe(gulp.dest('./dist'));

});

gulp.task('default', gulp.parallel('uglify'));