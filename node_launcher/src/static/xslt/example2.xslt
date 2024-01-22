<?xml version="1.0" encoding="UTF-8"?>
<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">

    <xsl:template match="/">
        <html>
            <head>
                <title>System Information</title>
            </head>
            <body>
                <h1>System Information</h1>

                <xsl:for-each select="//record">
                    <!-- Start of record -->
                    <h2>CPUs</h2>
                    <ul>
                        <!-- Calculate average, minimum, and maximum CPU usage -->
                        <xsl:variable name="cpuCount" select="count(cpus/cpu)" />
                        <xsl:variable name="totalCpuUsage" select="sum(cpus/cpu/@usage)" />
                        <xsl:variable name="avgCpuUsage" select="$totalCpuUsage div $cpuCount" />
                        <xsl:variable name="minCpuUsage" select="min(cpus/cpu/@usage)" />
                        <xsl:variable name="maxCpuUsage" select="max(cpus/cpu/@usage)" />

                        <!-- Calculate average, minimum, and maximum CPU frequency -->
                        <xsl:variable name="totalCpuFrequency" select="sum(cpus/cpu/@frequency)" />
                        <xsl:variable name="avgCpuFrequency"
                            select="$totalCpuFrequency div $cpuCount" />
                        <xsl:variable name="minCpuFrequency" select="min(cpus/cpu/@frequency)" />
                        <xsl:variable name="maxCpuFrequency" select="max(cpus/cpu/@frequency)" />

                        <li>Average CPU Usage: <xsl:value-of select="$avgCpuUsage" />%</li>
                        <li>Min CPU Usage ({$minCpuUsage}): <xsl:value-of
                                select="cpus/cpu[@usage = $minCpuUsage]/@name" /></li>
                        <li>Max CPU Usage ({$maxCpuUsage}): <xsl:value-of
                                select="cpus/cpu[@usage = $maxCpuUsage]/@name" /></li>
                        <li>Average CPU Frequency: <xsl:value-of select="$avgCpuFrequency" /> MHz</li>
                        <li>Min CPU Frequency ({$minCpuFrequency}): <xsl:value-of
                                select="cpus/cpu[@frequency = $minCpuFrequency]/@name" /></li>
                        <li>Max CPU Frequency ({$maxCpuFrequency}): <xsl:value-of
                                select="cpus/cpu[@frequency = $maxCpuFrequency]/@name" /></li>
                    </ul>

                    <h2>
        RAM</h2>
                    <!-- Your existing RAM section remains unchanged -->
                    <p>Total: <xsl:value-of select=".//ram/@total" /> bytes</p>
                    <p>Used: <xsl:value-of
                            select=".//ram/@used" /> bytes</p>
                    <p>Total Swap: <xsl:value-of
                            select=".//ram/@total_swap" /> bytes</p>
                    <p>Used Swap: <xsl:value-of
                            select=".//ram/@used_swap" /> bytes</p>

                    <h2>Disks</h2>
                    <ul>
                        <!-- Calculate total disk space, minimum and maximum available space -->
                        <xsl:variable name="totalDiskSpace" select="sum(disks/disk/@total_space)" />
                        <xsl:variable name="minAvailableSpace"
                            select="min(disks/disk/@available_space)" />
                        <xsl:variable name="maxAvailableSpace"
                            select="max(disks/disk/@available_space)" />

                        <li>Total Disk Space: <xsl:value-of select="$totalDiskSpace" /> bytes</li>
                        <li>Min Available Space ({$minAvailableSpace}): <xsl:value-of
                                select="disks/disk[@available_space = $minAvailableSpace]/@name" /></li>
                        <li>Max Available Space ({$maxAvailableSpace}): <xsl:value-of
                                select="disks/disk[@available_space = $maxAvailableSpace]/@name" /></li>
                    </ul>

                    <h2>
        Networks</h2>
                    <ul>
                        <!-- Your existing Networks section remains unchanged -->
                        <xsl:for-each select=".//network">
                            <li>
                                <xsl:value-of select="@name" />: Received <xsl:value-of
                                    select="@received" /> bytes, Total Received <xsl:value-of
                                    select="@total_received" /> bytes, Transmitted <xsl:value-of
                                    select="@transmitted" /> bytes, Total Transmitted <xsl:value-of
                                    select="@total_transmitted" /> bytes </li>
                        </xsl:for-each>
                    </ul>

                    <!-- End of record -->
                    <hr /> <!--
                    Separator between records -->
                </xsl:for-each>

            </body>
        </html>
    </xsl:template>

</xsl:stylesheet>