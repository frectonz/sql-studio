import { fetchTableData, fetchTables } from "@/api";
import { useQuery } from "@tanstack/react-query";
import { useEffect, useState } from "react";

export function useGetAllTables() {
    const query = useQuery({
        queryKey: ["tables", name],
        queryFn: () => fetchTables(),
    });

    return query
}

export function useGetTable() {

    const [tables, handleSetTables] = useState<string[]>([])
    const [columns, setColumns] = useState<string[]>([])

    const fetchData = async (data: string[]): Promise<void> => {

        const table = data.pop()
        if (!table) return

        const info = await fetchTableData(table, 1).catch(() => null)
        if (!info) return fetchData(data);

        setColumns(prev => {
            const isExist = prev.some((column) => info.columns.includes(column));
            if (isExist) return prev;
            return [...prev, ...info.columns];
        })

        return fetchData(data);
    }

    // useEffect(() => {

    //     if (!tables) return

    //     fetchData([...tables])

    // }, [tables])

    return {
        tables,
        columns,
        handleSetTables
    }
}