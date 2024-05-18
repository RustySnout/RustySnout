import React from 'react';
import styles from './layout.module.css';

const Table = ({ columns, rows }) => (
    <table className={styles.tableContainer}>
        <thead>
            <tr className={styles.headerRows}>
                {columns.map((column, index) => (
                    <th key={index} className={styles.headerCells}>{column}  </th>
                ))}
            </tr>
        </thead>
        <tbody>
            {rows.map((row, index) => (
                <tr key={index} className={styles.rows}>
                    {columns.map((column, colIndex) => (
                        <td key={colIndex} className={styles.cells}>{row[column]}  </td>
                    ))}
                </tr>
            ))}
        </tbody>
    </table>
);

export default Table;